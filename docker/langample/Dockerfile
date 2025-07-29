FROM eclipse-temurin:17-jre-alpine

ARG PATH_JAR

RUN test $PATH_JAR

ENV PATH_JAR $PATH_JAR

# Ktor uses 8080 by default
EXPOSE 8080

CMD java -jar $PATH_JAR
