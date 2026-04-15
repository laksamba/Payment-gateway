// TRON address validation
// Valid TRON addresses start with 'T' and are 34 characters long
export function isValidTronAddress(address: string): boolean {
  return /^T[a-zA-Z0-9]{33}$/.test(address);
}
