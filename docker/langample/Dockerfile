FROM eclipse-temurin:17-jre-alpine

ARG JAR_FILE

COPY ${JAR_FILE} /app/langample.jar

# Ktor uses 8080 by default
EXPOSE 8080

ENTRYPOINT ["java", "-jar", "/app/langample.jar"]
