FROM node:18-alpine

WORKDIR /app

RUN apk add --no-cache python3 make g++ linux-headers libusb-dev eudev-dev

COPY package.json package-lock.json ./
RUN npm ci

COPY build ./build

EXPOSE 3000

CMD ["npm", "run", "start:ssr", "--", "--host"]
