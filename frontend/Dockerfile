FROM node:16 as builder
ARG target_env
WORKDIR /app
COPY package.json .
RUN npm install
COPY . .
RUN cp ".env.${target_env}" ".env.local"
RUN npm run build

FROM nginx:stable
COPY --from=builder /app/build /usr/share/nginx/html
