package utils

import "time"

const millisInSecond = 1000
const nsInSecond = 1000000

// Converts Unix Epoch from milliseconds to time.Time
// see https://github.com/Tigraine/go-timemilli
func FromUnixMilli(ms int64) time.Time {
	return time.Unix(ms/int64(millisInSecond), (ms%int64(millisInSecond))*int64(nsInSecond))
}
