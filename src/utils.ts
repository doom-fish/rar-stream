export function groupBy<T>(arr: T[], fn: (item: T) => any) {
  return arr.reduce<Record<string, T[]>>((prev, curr) => {
    const groupKey = fn(curr);
    const group = prev[groupKey] || [];
    group.push(curr);
    return { ...prev, [groupKey]: group };
  }, {});
}

export function sum(arr: number[]) {
  return arr.reduce((s, n) => s + n);
}
export function mapValues<T extends Object, S>(
  object: T,
  mapper: (value: T[keyof T]) => S
) {
  return Object.fromEntries(
    Object.entries(object).map(([key, value]) => [key, mapper(value)])
  ) as { [key in keyof T]: S };
}
