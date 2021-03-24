import { createClient, RedisClient } from 'redis';
import { promisify } from 'util';

// eslint-disable-next-line import/prefer-default-export
export class RedisCache {
  private client: RedisClient;

  private redisSet: (key: string, value: string) => Promise<void>;

  private redisGet: (key: string) => Promise<string | null>;

  constructor(redisUrl = 'redis://localhost:6379') {
    this.client = createClient({ url: redisUrl });

    this.redisSet = promisify<string, string>(this.client.set).bind(this.client);
    this.redisGet = promisify(this.client.get).bind(this.client);
  }

  public async get(key: string): Promise<string> {
    const value = await this.redisGet(key);
    if (!value) {
      return '';
    }
    return value;
  }

  public async set(key: string, value: string): Promise<void> {
    this.redisSet(key, value);
  }

  public close(): void {
    this.client.quit();
  }
}
