FROM logstash:7.11.2

RUN bin/logstash-plugin install logstash-output-influxdb

COPY ./config/logstash.yml /usr/share/logstash/config/logstash.yml
COPY ./config/pipelines.yml /usr/share/logstash/config/pipelines.yml
COPY ./pipeline/ /usr/share/logstash/pipeline/
