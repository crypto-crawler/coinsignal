import { Publisher } from 'utils';
import Web3 from 'web3';
import { BlockHeader } from 'web3-eth';
import yargs from 'yargs';
import { REDIS_TOPIC_ETH_BLOCK_HEADER } from './common';

const web3 = new Web3(
  new Web3.providers.WebsocketProvider(process.env.FULL_NODE_URL || 'ws://localhost:8546'),
);

const commandModule: yargs.CommandModule = {
  command: 'crawler_block_header',
  describe: 'Crawl Ethereum block headers',
  // eslint-disable-next-line no-shadow
  builder: (yargs) => yargs.options({}),
  handler: async () => {
    const publisher = new Publisher<BlockHeader>(process.env.REDIS_URL || 'redis://localhost:6379');

    web3.eth.subscribe('newBlockHeaders', (error, blockHeader) => {
      if (error) {
        console.error(error);
      } else if (blockHeader) {
        publisher.publish(REDIS_TOPIC_ETH_BLOCK_HEADER, blockHeader);
      }
    });
  },
};

export default commandModule;
