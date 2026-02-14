FROM node:20-alpine

RUN apk add --no-cache python3 make g++ linux-headers libusb-dev eudev-dev

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm install --production

COPY build ./build

EXPOSE 3000

CMD ["npm", "run", "start:ssr", "--", "--host"]
