/** A number and the name of what it counts, such as "1 Essay" or "3 Passages". */
export const countLabel = (n: number, noun: string): string => `${n} ${n === 1 ? noun : `${noun}s`}`;
