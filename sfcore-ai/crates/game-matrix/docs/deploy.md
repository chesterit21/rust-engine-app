Jalankan dari root sfcore-ai yang ada Dockerfile nya :

```
docker stop game-matrix
docker rm game-matrix

docker build -t game-matrix:latest .

docker run -d \
  --name game-matrix \
  -p 3001:3001 \
  -e DATABASE_URL=/home/sfcore/server-db/gamesmatrix.db \
  -v /home/sfcore/server-db:/home/sfcore/server-db \
  game-matrix:latest
  
``
