package utils

import (
	"context"
	"log"
	"time"

	"github.com/go-redis/redis/v8"
)

func pingRedis(ctx context.Context, redis_url string) bool {
	client := redis.NewClient(&redis.Options{
		Addr: redis_url,
	})

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
