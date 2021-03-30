package utils

import (
	"context"
	"log"
	"time"

	influxdb2 "github.com/influxdata/influxdb-client-go/v2"
)

func WaitInfluxDB(ctx context.Context, client influxdb2.Client) {
	for {
		ready, _ := client.Ready(ctx)
		if ready {
			break
		} else {
			log.Println("InfluxDB is not ready, sleeping for 3 seconds")
			time.Sleep(3 * time.Second)
		}
	}
}
