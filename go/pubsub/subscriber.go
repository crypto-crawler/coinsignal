package pubsub

import (
	"context"

	"github.com/go-redis/redis/v8"
)

type Subscriber struct {
	pubsub *redis.PubSub
	on_msg func(string)
}

func NewSubscriber(ctx context.Context, redis_url, channel string, on_msg func(string)) *Subscriber {
	rdb := redis.NewClient(&redis.Options{
		Addr: redis_url,
	})
	pubsub := rdb.Subscribe(ctx, channel)

	return &Subscriber{pubsub, on_msg}
}

func (subscriber *Subscriber) Run() {
	ch := subscriber.pubsub.Channel()
	// Consume messages.
	for msg := range ch {
		// fmt.Println(msg.Channel, msg.Payload)
		subscriber.on_msg(msg.Payload)
	}
}

func (subscriber *Subscriber) Close() {
	subscriber.pubsub.Close()
}
