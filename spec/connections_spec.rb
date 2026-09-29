# frozen_string_literal: true

require 'spec_helper'

RSpec.describe 'Base connector connections' do
  let(:wasm_path) do
    File.expand_path('../target/wasm32-wasip2/release/base_connector.wasm', __dir__)
  end
  let(:app) { AppBridge::App.new(wasm_path) }

  before(:all) do
    root = File.expand_path('..', __dir__)
    ok = system(
      { 'CARGO_TARGET_DIR' => File.join(root, 'target') },
      'cargo', 'build', '--target', 'wasm32-wasip2', '--release',
      chdir: root
    )
    raise 'Failed to build base_connector.wasm for connections spec' unless ok
  end

  describe 'connection-config' do
    it 'reports WIT 5 and connections support' do
      expect(app.wit_version).to eq('5.0.0')
      expect(app.connections_supported?).to be(true)
    end

    it 'returns disabled connection config by default' do
      config = JSON.parse(app.connection_config)

      expect(config['strategies']).to eq([])
      expect(config['default_strategy']).to eq('')
      expect(config['runtime']).to eq({})
    end
  end
end
