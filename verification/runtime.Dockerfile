FROM ruby:3.4.12-slim-bookworm
RUN gem install app_bridge -v 5.0.0 --no-document \
 && gem install json_schemer -v 2.5.0 --no-document
WORKDIR /work
USER 65534:65534
ENTRYPOINT ["/usr/bin/env", "-i", "PATH=/usr/local/bin:/usr/bin:/bin", "GEM_HOME=/usr/local/bundle", "GEM_PATH=/usr/local/bundle:/usr/local/lib/ruby/gems/3.4.0", "LANG=C.UTF-8"]
