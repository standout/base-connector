# frozen_string_literal: true

require 'app_bridge'
require 'rspec-benchmark'
require 'json'
require 'net/http'
require 'uri'
require_relative 'test_helper'

RSpec.configure do |config|
  # Enable flags like --only-failures and --next-failure
  config.example_status_persistence_file_path = '.rspec_status'

  # Disable RSpec exposing methods globally on `Module` and `main`
  config.disable_monkey_patching!

  config.expect_with :rspec do |c|
    c.syntax = :expect
  end

  config.include RSpec::Benchmark::Matchers

  # Mock server configuration
  config.before(:suite) do
    @mock_server = TestHelper.create_mock_server
    @mock_server.start
  end

  config.after(:suite) do
    @mock_server&.stop
  end

  # Provide mock server to tests
  config.before(:each) do
    @mock_server ||= TestHelper.create_mock_server
  end
end
