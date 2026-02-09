FROM node:18.13.0-alpine AS builder

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM node:18.13.0-alpine

WORKDIR /app

COPY --from=builder /app/package.json /app/package-lock.json ./
RUN npm ci --omit=dev

COPY --from=builder /app/build ./build
COPY --from=builder /app/node_modules/@lomray/vite-ssr-boost ./node_modules/@lomray/vite-ssr-boost

EXPOSE 5173

CMD ["npm", "run", "start:ssr"]
