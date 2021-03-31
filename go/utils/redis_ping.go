package utils

import (
	"context"
	"log"
	"time"

	"github.com/go-redis/redis/v8"
)

func NewRedisClient(redis_url string) *redis.Client {
  opt, err := redis.ParseURL(redis_url)
	if err != nil {
		log.Fatalln(err)
	}

	return redis.NewClient(opt)
}

func pingRedis(ctx context.Context, redis_url string) bool {
	client := NewRedisClient(redis_url)

	_, err := client.Ping(ctx).Result()
	return err == nil
}

func WaitRedis(ctx context.Context, redis_url string) {
	for {
		ok := pingRedis(ctx, redis_url)
		if ok {
			break
		} else {
			log.Println("Redis is not ready, sleeping for 1 second")
			time.Sleep(1 * time.Second)
		}
	}
}
