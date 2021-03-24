import { Publisher, Subscriber } from 'utils';
import web3 from 'web3';
import WebSocket from 'ws';
import yargs from 'yargs';
import { REDIS_TOPIC_ETH_GAS_PRICE, REDIS_TOPIC_ETH_PRICE } from './common';

interface GasPriceMsg {
  rapid: number;
  fast: number;
  standard: number;
  slow: number;
  timestamp: number;
}

interface WebsocketMsg {
  type: string;
  data: {
    gasPrices: {
      rapid: number;
      fast: number;
      standard: number;
      slow: number;
    };
    timestamp: number;
  };
}

const commandModule: yargs.CommandModule = {
  command: 'crawler_gas_price',
  describe: 'Crawl Ethereum gas price from gasnow.org',
  // eslint-disable-next-line no-shadow
  builder: (yargs) => yargs.options({}),
  handler: async () => {
    const gasLimit = 21000;
    let ethPrice = -1;
    const subscriberEthPrice = new Subscriber<number>(
      async (price): Promise<void> => {
        ethPrice = price; // update ETH price
      },
      REDIS_TOPIC_ETH_PRICE,
      process.env.REDIS_URL || 'redis://localhost:6379',
    );
    subscriberEthPrice.run();

    while (ethPrice < 0) {
      // eslint-disable-next-line no-await-in-loop
      await new Promise((resolve) => setTimeout(resolve, 3000));
    }

    const publisher = new Publisher<GasPriceMsg>(process.env.REDIS_URL || 'redis://localhost:6379');

    // see https://taichi.network/#gasnow
    const ws = new WebSocket('wss://www.gasnow.org/ws');

    ws.on('message', function incoming(data) {
      const msg: WebsocketMsg = JSON.parse(data as string);
      const gasPrice: GasPriceMsg = {
        ...msg.data.gasPrices,
        timestamp: msg.data.timestamp,
      };

      gasPrice.rapid =
        parseFloat(web3.utils.fromWei((gasPrice.rapid * gasLimit).toString(), 'ether')) * ethPrice;
      gasPrice.fast =
        parseFloat(web3.utils.fromWei((gasPrice.fast * gasLimit).toString(), 'ether')) * ethPrice;
      gasPrice.standard =
        parseFloat(web3.utils.fromWei((gasPrice.standard * gasLimit).toString(), 'ether')) *
        ethPrice;
      gasPrice.slow =
        parseFloat(web3.utils.fromWei((gasPrice.slow * gasLimit).toString(), 'ether')) * ethPrice;

      publisher.publish(REDIS_TOPIC_ETH_GAS_PRICE, gasPrice);
    });
  },
};

export default commandModule;
