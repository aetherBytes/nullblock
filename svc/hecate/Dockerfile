FROM node:18.13.0-alpine AS builder

WORKDIR /app

RUN apk add --no-cache python3 make g++ linux-headers libusb-dev eudev-dev

COPY package.json package-lock.json ./
RUN npm install

COPY . .
RUN npm run build

FROM node:18.13.0-alpine

RUN apk add --no-cache python3 make g++ linux-headers libusb-dev eudev-dev

WORKDIR /app

COPY --from=builder /app/package.json /app/package-lock.json ./
RUN npm install --production

COPY --from=builder /app/build ./build
COPY --from=builder /app/node_modules/@lomray/vite-ssr-boost ./node_modules/@lomray/vite-ssr-boost

EXPOSE 3000

CMD ["npm", "run", "start:ssr", "--", "--host"]
