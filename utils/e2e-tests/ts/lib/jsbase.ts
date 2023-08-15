export type ArrayMapFn<
  T extends readonly any[],
  Index extends keyof T,
  Result
> = (item: T[Index], index: Index, src: T) => Result;

export type ArrayMapResult<
  T extends readonly any[],
  F extends ArrayMapFn<T, any, any>
> = {
  [K in keyof T]: F extends ArrayMapFn<T, K, infer R> ? R : never;
};

export function arrayMap<
  const T extends readonly any[],
  const F extends ArrayMapFn<T, any, any>
>(array: T, fn: F): ArrayMapResult<T, F> {
  return array.map(fn as any) as ArrayMapResult<T, F>;
}

export function objectKeysMap<
  const T extends { [Keys in any]: any },
  F extends ArrayMapFn<(keyof T)[], any, any>
>(object: T, fn: F): ArrayMapResult<(keyof T)[], F> {
  return arrayMap(Object.keys(object) as (keyof T)[], fn);
}

export function objectValuesMap<
  const T extends { [Keys in any]: any },
  F extends ArrayMapFn<T[keyof T][], any, any>
>(object: T, fn: F): ArrayMapResult<T[keyof T][], F> {
  return arrayMap(Object.values(object) as T[keyof T][], fn);
}
