FROM debian:bookworm-slim

ARG BIN_FILE
ARG API_KEY_CHATGPT

RUN test $API_KEY_CHATGPT

ENV API_KEY_CHATGPT $API_KEY_CHATGPT

COPY ${BIN_FILE} /app/backend
RUN chmod +x /app/backend

EXPOSE 8080

CMD /app/backend --api-key-chatgpt $API_KEY_CHATGPT
