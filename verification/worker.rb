# frozen_string_literal: true

# This file is executed only inside the restricted verification container.
require 'json'
require 'digest'
require 'socket'
require 'timeout'
require 'app_bridge'
require 'json_schemer'

STDOUT.sync = true
wasm, expected_digest, scenario = ARGV
abort 'Unexpected environment' unless (ENV.keys - %w[PATH GEM_HOME GEM_PATH LANG]).empty?
# Docker Desktop exposes inactive tunnel devices; only loopback may be UP.
abort 'External interface active' unless Socket.getifaddrs.all? { |i| i.name == 'lo' || (i.flags & Socket::IFF_UP).zero? }
abort 'External route present' unless File.readlines('/proc/net/route').drop(1).all? { |line| line.strip.empty? }
abort 'WASM is writable' if File.writable?(wasm)
actual_digest = Digest::SHA256.file(wasm).hexdigest
abort 'Artifact digest mismatch before load' unless actual_digest == expected_digest
abort 'Wrong App Bridge' unless AppBridge::VERSION == '5.0.0'
app = AppBridge::App.new(wasm, environment_variables: {})
connection = AppBridge::Connection.new('synthetic-fixture', 'No credentials', '{}')
context = lambda do |action, input|
  AppBridge::ActionContext.new(action, connection, input)
end
execute = lambda do |action, input|
  JSON.parse(app.execute_action(context.call(action, JSON.generate(input))).serialized_output)
end
checks = []
case scenario
when 'contract', 'candidate'
  expected_actions = scenario == 'candidate' ? ['echo_contract'] : %w[cpu_probe echo_contract memory_probe network_probe]
  raise 'Unexpected actions' unless app.action_ids.sort == expected_actions
  raise 'Unexpected triggers' unless app.trigger_ids.empty?
  raise 'Wrong WIT' unless app.wit_version == '5.0.0'
  raise 'Connection config not empty' unless JSON.parse(app.connection_config) == {
    'strategies' => [], 'default_strategy' => '', 'runtime' => {}
  }
  ctx = context.call('echo_contract', '{}')
  schemas = {
    'input' => JSON.parse(app.action_input_schema(ctx)),
    'output' => JSON.parse(app.action_output_schema(ctx))
  }
  schemas.each do |direction, schema|
    expected = JSON.parse(File.read("/verification/schemas/#{direction}.json"))
    raise 'Embedded contract differs from source contract' unless schema == expected
  end
  input = { 'reference' => 'SYNTHETIC-42', 'values' => [false, nil, 125, 1.0, -9007199254740991, 9007199254740991] }
  expected = input.merge('environment_count' => 0)
  raise 'Input schema rejected sample' unless JSONSchemer.schema(schemas['input']).valid?(input)
  first = execute.call('echo_contract', input)
  raise 'Incorrect deterministic output' unless first == expected && execute.call('echo_contract', input) == first
  raise 'Output schema mismatch' unless JSONSchemer.schema(schemas['output']).valid?(first)
  checks.concat(%w[discovery wit_version empty_connections embedded_schemas deterministic_output false_null])
  checks << (scenario == 'candidate' ? 'constant_environment_count' : 'empty_guest_environment')
  [{}, input.merge('extra' => true), input.merge('values' => ["invalid"]), input.merge('reference' => ''), input.merge('values' => [1.5]), input.merge('values' => [9007199254740992])].each do |invalid|
    raise 'Invalid input accepted by schema' if JSONSchemer.schema(schemas['input']).valid?(invalid)
    begin
      execute.call('echo_contract', invalid)
    rescue StandardError => e
      raise unless e.message.include?('Invalid fixture input')
      next
    end
    raise 'Invalid input accepted by real WASM'
  end
  begin
    app.execute_action(context.call('echo_contract', '{'))
  rescue StandardError => e
    raise unless e.message.include?('Invalid fixture input')
    checks << 'malformed_json_rejected'
  end
  raise 'Malformed JSON accepted' unless checks.include?('malformed_json_rejected')
  begin
    execute.call('not_an_action', {})
  rescue StandardError => e
    raise unless e.message.include?('Unknown fixture action')
    checks << 'unknown_action_rejected'
  end
  raise 'Unknown action accepted' unless checks.include?('unknown_action_rejected')
  if scenario == 'candidate'
    supplied_connection = AppBridge::Connection.new('synthetic', 'Forbidden data', '{"unexpected":true}')
    begin
      app.execute_action(AppBridge::ActionContext.new('echo_contract', supplied_connection, JSON.generate(input)))
    rescue StandardError => e
      raise unless e.message.include?('No connection data permitted')
      checks << 'connection_data_rejected'
    end
    raise 'Connection data accepted' unless checks.include?('connection_data_rejected')
    %w[network_probe cpu_probe memory_probe].each do |forbidden|
      [:execute_action, :action_input_schema, :action_output_schema].each do |operation|
        begin
          app.public_send(operation, context.call(forbidden, '{}'))
        rescue StandardError => e
          raise unless e.message.include?('Unknown fixture action')
          next
        end
        raise 'Diagnostic entry point accepted by candidate'
      end
    end
    checks << 'diagnostic_actions_absent_and_rejected'
  end
  checks << 'invalid_inputs_rejected'
when 'network'
  begin
    Timeout.timeout(2) { TCPSocket.new('192.0.2.1', 80).close }
  rescue Errno::ENETUNREACH, Errno::EHOSTUNREACH
    checks << 'host_socket_network_denied'
  end
  raise 'Network boundary not demonstrated' unless checks.include?('host_socket_network_denied')
  raise 'WASM HTTP unexpectedly succeeded' unless execute.call('network_probe', {}) == { 'network_denied' => true }
  checks << 'real_wasm_http_denied'
when 'cpu', 'memory'
  puts JSON.generate(phase: 'probe_started', scenario:, artifact_sha256: actual_digest)
  execute.call("#{scenario}_probe", {})
  raise 'Resource probe unexpectedly returned'
else
  abort 'Unknown scenario'
end
raise 'Artifact changed during execution' unless Digest::SHA256.file(wasm).hexdigest == actual_digest
puts JSON.generate(status: 'passed', scenario:, artifact_sha256: actual_digest,
                   app_bridge: AppBridge::VERSION, wit_version: app.wit_version,
                   ruby: RUBY_VERSION, checks:, schemas: defined?(schemas) ? schemas : nil)
