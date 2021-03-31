package pubsub

import (
	"context"

	"github.com/ethereum/go-ethereum/log"
	"github.com/go-redis/redis/v8"
	"github.com/soulmachine/coinsignal/utils"
)

type Publisher struct {
	rdb *redis.Client
	ctx context.Context
}

func NewPublisher(ctx context.Context, redis_url string) *Publisher {
	rdb := utils.NewRedisClient(redis_url)
	return &Publisher{rdb, ctx}
}

func (publisher *Publisher) Publish(channel, msg string) {
	err := publisher.rdb.Publish(publisher.ctx, channel, msg).Err()
	if err != nil {
		log.Error(err.Error())
	}
}

func (publisher *Publisher) Close() {
	publisher.rdb.Close()
}
