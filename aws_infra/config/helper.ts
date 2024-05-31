import { devConfig } from './dev';
import { Config, Scope } from './type';

const configMap = new Map<Scope, Config>([['dev', devConfig]]);

export function isScope(value: any): value is Scope {
  return value === 'prod' || value === 'dev';
}

export function getConfig(scope: Scope): Config {
  const config = configMap.get(scope);
  if (!config) {
    throw new Error(`No config configured for scope ${scope}`);
  }
  return config;
}

export function stackId(scope: Scope, stackId: string): string {
  return `${scope}${stackId}`;
}
