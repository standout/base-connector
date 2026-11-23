# frozen_string_literal: true

source 'https://rubygems.org'

ruby '3.4.8'

# Use the WIT v6 source until app_bridge 6.0.0 is published on RubyGems.
gem 'app_bridge', '~> 6.0', git: 'https://github.com/standout/app_bridge.git',
                            ref: '1c9b523f186acfad65f39b2e6804fc43b3d8a063'

# RSpec for testing
gem 'rspec', '~> 3.12'
gem 'rspec-benchmark', '~> 0.6'

group :development, :test do
  gem 'rubocop', '~> 1.50'
  gem 'rubocop-rspec', '~> 2.20'
end
