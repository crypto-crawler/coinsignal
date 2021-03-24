/* eslint-disable camelcase */
import CoinMarketCap from 'coinmarketcap-node';
import { Publisher } from 'utils';
import { REDIS_TOPIC_CMC_GLOBAL_METRICS } from './common';

interface CMCMetrics {
  active_cryptocurrencies: number;
  total_cryptocurrencies: number;
  active_market_pairs: number;
  active_exchanges: number;
  total_exchanges: number;
  eth_dominance: number;
  btc_dominance: number;
  defi_volume_24h: number;
  defi_volume_24h_reported: number;
  defi_market_cap: number;
  defi_24h_percentage_change: number;
  stablecoin_volume_24h: number;
  stablecoin_volume_24h_reported: number;
  stablecoin_market_cap: number;
  stablecoin_24h_percentage_change: number;
  derivatives_volume_24h: number;
  derivatives_volume_24h_reported: number;
  derivatives_24h_percentage_change: number;
  total_market_cap: number;
  total_volume_24h: number;
  total_volume_24h_reported: number;
  altcoin_volume_24h: number;
  altcoin_volume_24h_reported: number;
  altcoin_market_cap: number;
  timestamp: number;
  last_updated?: string;
  quote?: unknown;
}

async function fetchCMCMtrics(): Promise<CMCMetrics> {
  if (!process.env.CMC_API_KEY) {
    throw new Error('CMC_API_KEY is not set');
  }
  const cmc = new CoinMarketCap(process.env.CMC_API_KEY);
  const metrics = await cmc.fetchLatestGlobalMetrics({});
  const cmcMetrics: CMCMetrics = {
    ...metrics,
    ...metrics.quote.USD,
    timestamp: new Date(metrics.last_updated).getTime(),
  };

  delete cmcMetrics.quote;
  delete cmcMetrics.last_updated;

  return cmcMetrics;
}

(async () => {
  const publisher = new Publisher<CMCMetrics>(process.env.REDIS_URL || 'redis://localhost:6379');
  const metrics = await fetchCMCMtrics();
  publisher.publish(REDIS_TOPIC_CMC_GLOBAL_METRICS, metrics);
  publisher.close();
})();
