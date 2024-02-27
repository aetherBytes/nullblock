FROM node:18.13.0-alpine

ARG RUN_TYPE=ssr
ARG ENV_MODE=production
ARG WEB_PATH=/var/www
ARG BUILD_PATH=./build
ARG SERVICE_PATH=./

ENV NODE_ENV=${ENV_MODE}
ENV TYPE=${RUN_TYPE}

RUN mkdir -p $WEB_PATH


COPY ${BUILD_PATH} $WEB_PATH/build
COPY $SERVICE_PATH/package.json $WEB_PATH/package.json
COPY $SERVICE_PATH/package-lock.json $WEB_PATH/package-lock.json

WORKDIR $WEB_PATH

RUN npm ci --omit=dev

CMD npm run start:${TYPE} -- --host
