#!/usr/bin/env node
import yargs from 'yargs';
import crawlerBlockHeader from './crawlers/crawler_block_header';
import crawlerETHPriceModule from './crawlers/crawler_eth_price';
import crawlerGasPriceModule from './crawlers/crawler_gas_price';
import ethMinerRevenueModule from './transform/eth_miner_revenue';

// eslint-disable-next-line no-unused-expressions
yargs
  .command(crawlerBlockHeader)
  .command(crawlerETHPriceModule)
  .command(crawlerGasPriceModule)
  .command(ethMinerRevenueModule)
  .wrap(null)
  .demandCommand(1, '').argv;
