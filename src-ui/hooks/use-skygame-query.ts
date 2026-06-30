import { useEffect, useState, type DependencyList } from "react";

export function useSkyGameQuery<T>(
  load: () => Promise<T>,
  dependencies: DependencyList,
  initialValue: T,
) {
  const [value, setValue] = useState(initialValue);

  useEffect(() => {
    let cancelled = false;

    void load().then((nextValue) => {
      if (!cancelled) {
        setValue(nextValue);
      }
    });

    return () => {
      cancelled = true;
    };
  }, dependencies);

  return value;
}
