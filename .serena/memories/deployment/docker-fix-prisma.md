Fixed Prisma initialization error in Dockerfile by ensuring node_modules containing the generated Prisma Client are copied from the build stage to the runtime stage.

Instructions for user to re-run:
1. docker compose up --build -d
2. Run migrations/seed if needed:
   docker exec -it webph-app npx prisma migrate deploy
   docker exec -it webph-app npx prisma db seed