# frozen_string_literal: true

source 'https://rubygems.org'

ruby '3.4.8'

# Use the WIT v6 source until app_bridge 5.1.0 is published on RubyGems.
gem 'app_bridge', '~> 5.1', git: 'https://github.com/standout/app_bridge.git',
                            ref: 'd7260c3aab577500156ce24799328382ea1996f9'

# RSpec for testing
gem 'rspec', '~> 3.12'
gem 'rspec-benchmark', '~> 0.6'

group :development, :test do
  gem 'rubocop', '~> 1.50'
  gem 'rubocop-rspec', '~> 2.20'
end
