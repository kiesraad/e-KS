FROM mcr.microsoft.com/playwright:v1.58.2-noble
WORKDIR /app
COPY package*.json /app/
COPY playwright.config.ts /app/playwright.config.ts
RUN npm ci

CMD ["npx", "playwright", "test", "--project=chromium", "--project=firefox", "--project=webkit", "--workers=3"]
