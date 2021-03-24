import Axios from 'axios';
import { Publisher, Subscriber } from 'utils';
import web3 from 'web3';
import { BlockHeader } from 'web3-eth';
import yargs from 'yargs';
import {
  REDIS_TOPIC_ETH_BLOCK_HEADER,
  REDIS_TOPIC_ETH_MINER_REVENUE,
  REDIS_TOPIC_ETH_PRICE,
} from '../crawlers/common';

interface BlockRewardMsg {
  number: number;
  timestamp: number;
  miner: string;
  reward: number;
  // eslint-disable-next-line camelcase
  reward_usd: number;
}

// Deprecated due to Infura API limit
// async function calcBlockReward(blockHash: string): Promise<number> {
//   const block = await web3.eth.getBlock(blockHash);
//   const txs = await Promise.all(
//     block.transactions.map((txHash) => web3.eth.getTransaction(txHash)),
//   );
//   const tmp = txs.filter((tx) => tx).map((tx) => web3.utils.fromWei(tx.value, 'ether'));
//   const blockReward = _.sum(tmp.map((x) => parseFloat(x)));
//   return blockReward;
// }

async function fetchBlockReward(
  blockNumber: number,
  ethPrice: number,
): Promise<BlockRewardMsg | undefined> {
  for (let i = 0; i < 3; i += 1) {
    // eslint-disable-next-line no-await-in-loop
    await new Promise((resolve) => setTimeout(resolve, 5000));
    // eslint-disable-next-line no-await-in-loop
    const response = await Axios.get(
      `https://api.etherscan.io/api?module=block&action=getblockreward&blockno=${blockNumber}&apikey=${process.env.ETHERSCAN_API_KEY}`,
    );
    if (response.data.result.blockNumber) {
      const raw = response.data.result as {
        blockNumber: string;
        timeStamp: string;
        blockMiner: string;
        blockReward: string;
      };
      const ethNum = parseFloat(web3.utils.fromWei(raw.blockReward, 'ether'));
      const reward: BlockRewardMsg = {
        number: blockNumber,
        timestamp: parseInt(raw.timeStamp, 10),
        miner: raw.blockMiner,
        reward: parseInt(raw.blockReward, 10),
        // eslint-disable-next-line camelcase
        reward_usd: ethPrice * ethNum,
      };
      return reward;
    }
  }
  console.warn('Failed to fetch ', blockNumber);
  return undefined;
}

const commandModule: yargs.CommandModule = {
  command: 'eth_miner_revenue',
  describe: 'Convert Ethereum miner revenue from ETH to USD.',
  // eslint-disable-next-line no-shadow
  builder: (yargs) => yargs.options({}),
  handler: async () => {
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

    const publisher = new Publisher<BlockRewardMsg>(
      process.env.REDIS_URL || 'redis://localhost:6379',
    );

    const subscriber = new Subscriber<BlockHeader>(
      async (blockHeader): Promise<void> => {
        const blockReward = await fetchBlockReward(blockHeader.number, ethPrice);
        if (blockReward) {
          publisher.publish(REDIS_TOPIC_ETH_MINER_REVENUE, blockReward);
        }
      },
      REDIS_TOPIC_ETH_BLOCK_HEADER,
      process.env.REDIS_URL || 'redis://localhost:6379',
    );

    subscriber.run();
  },
};

export default commandModule;
