/**
 * Money type - represents monetary values in cents to avoid floating point issues.
 *
 * All prices are stored and transmitted as integers (cents).
 * Display formatting is handled by utility functions.
 */

/** Money value in cents (CAD) */
export type Cents = number & { readonly _brand: 'Cents' };

/** Create a Cents value from a number */
export function cents(value: number): Cents {
    if (!Number.isInteger(value)) {
        throw new Error(`Money must be an integer (cents), got: ${value}`);
    }
    return value as Cents;
}

/** Convert dollars to cents */
export function dollarsToCents(dollars: number): Cents {
    return cents(Math.round(dollars * 100));
}

/** Convert cents to dollars (for display only) */
export function centsToDollars(amount: Cents): number {
    return amount / 100;
}

/** Format cents as currency string */
export function formatMoney(amount: Cents, currency: string = 'CAD'): string {
    const dollars = centsToDollars(amount);
    return new Intl.NumberFormat('en-CA', {
        style: 'currency',
        currency,
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
    }).format(dollars);
}

/** Format cents as short currency (no decimals if whole dollar) */
export function formatMoneyShort(amount: Cents, currency: string = 'CAD'): string {
    const dollars = centsToDollars(amount);
    const isWholeDollar = amount % 100 === 0;
    return new Intl.NumberFormat('en-CA', {
        style: 'currency',
        currency,
        minimumFractionDigits: isWholeDollar ? 0 : 2,
        maximumFractionDigits: 2,
    }).format(dollars);
}

/** Pricing constants (in cents) */
export const PRICING = {
    FREE: cents(0),
    MONTHLY: cents(1999), // $19.99/month
    ANNUAL: cents(14999), // $149.99/year
    ANNUAL_MONTHLY_EQUIVALENT: cents(1250), // ~$12.50/month equivalent
} as const;

/** Calculate savings for annual plan */
export function calculateAnnualSavings(): Cents {
    const monthlyAnnual = PRICING.MONTHLY * 12;
    return cents(monthlyAnnual - PRICING.ANNUAL);
}

/** Calculate savings percentage for annual plan */
export function calculateAnnualSavingsPercent(): number {
    const monthlyAnnual = PRICING.MONTHLY * 12;
    return Math.round(((monthlyAnnual - PRICING.ANNUAL) / monthlyAnnual) * 100);
}
