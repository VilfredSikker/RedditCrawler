# RedditCrawler
A Reddit crawler to look up subreddits with different parameters


## Setup
Starting the containers is done with docker-compose

### Development mode on port 3000
```docker-compose up --build```
build is required when packages are updated. Not for src changes.
src folder is synced with volumes, which should hot-reload new changes

### Prod mode on port 80
```docker-compose -f docker-compose.prod.yml up --build```

### Individual Docker Images

#### Webservice
_Build & run image_: 
1) go to /webservice/ 
2) docker build -t reddit_crawler_webservice .
3) docker run --rm -it reddit_crawler_webservice
