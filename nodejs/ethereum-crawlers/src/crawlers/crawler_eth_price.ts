import { Publisher } from 'utils';
import WebSocket from 'ws';
import yargs from 'yargs';
import { REDIS_TOPIC_ETH_PRICE } from './common';

const commandModule: yargs.CommandModule = {
  command: 'crawler_eth_price',
  describe: 'Crawl ETH mark price from Binance',
  // eslint-disable-next-line no-shadow
  builder: (yargs) => yargs.options({}),
  handler: async () => {
    const publisher = new Publisher<number>(process.env.REDIS_URL || 'redis://localhost:6379');

    // see https://binance-docs.github.io/apidocs/futures/en/#mark-price-stream
    const ws = new WebSocket('wss://fstream.binance.com/ws/ethusdt@markPrice');
    ws.on('message', function incoming(data) {
      const msg = JSON.parse(data as string);
      const price = parseFloat(msg.p);
      publisher.publish(REDIS_TOPIC_ETH_PRICE, price);
    });
  },
};

export default commandModule;
