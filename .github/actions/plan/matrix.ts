export const matrixItems = <T extends Record<string, any>>(
  map: T
): Array<T[keyof T]> => Object.values(map);

export const matrixItemsFiltered = <T extends Record<string, any>>(
  map: T,
  predicate: <K extends keyof T>(item: T[K]) => boolean
): Array<T[keyof T]> => matrixItems<T>(map).filter(predicate);

export const evalMatrix = <Keys extends string>(
  dimensions: Record<Keys, Array<any>>,
  includes: Array<Record<Keys, any>>
): Array<Record<Keys, any>> => {
  const evalNext = (
    allVariants: Array<Partial<Record<Keys, any>>>,
    key: Keys,
    values: Array<any>
  ) =>
    allVariants.flatMap((variant) =>
      values.map((value) => ({ ...variant, [key]: value }))
    );
  const dimensionKeys = Object.keys(dimensions) as Array<
    keyof typeof dimensions
  >;
  const evaluated = dimensionKeys.reduce(
    (allVariants, dimensionKey) =>
      evalNext(allVariants, dimensionKey, dimensions[dimensionKey]),
    [{}] as Array<Partial<Record<Keys, any>>>
  ) as Array<Record<Keys, any>>;
  return [...evaluated, ...includes];
};

export const logMatrix = (matrix: any) =>
  console.log(JSON.stringify(matrix, null, "  "));
