FROM eclipse-temurin:17-jre-alpine

ARG JAR_FILE
ARG API_KEY_CHATGPT

RUN test $API_KEY_CHATGPT

ENV API_KEY_CHATGPT $API_KEY_CHATGPT

COPY ${JAR_FILE} /app/langample.jar

# Ktor uses 8080 by default
EXPOSE 8080

CMD java -jar /app/langample.jar --api-key-chat-gpt $API_KEY_CHATGPT