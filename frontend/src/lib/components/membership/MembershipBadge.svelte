<script lang="ts">
    import type { MembershipTier, MembershipStatus } from '../../types';
    import { formatStatus, getStatusColor } from '../../types';

    interface Props {
        tier: MembershipTier;
        status?: MembershipStatus;
        size?: 'sm' | 'md' | 'lg';
    }

    let { tier, status, size = 'md' }: Props = $props();

    const tierColors: Record<MembershipTier, string> = {
        free: 'bg-gray-100 text-gray-700',
        monthly: 'bg-blue-100 text-blue-700',
        annual: 'bg-purple-100 text-purple-700',
    };

    const tierNames: Record<MembershipTier, string> = {
        free: 'Free',
        monthly: 'Monthly',
        annual: 'Annual',
    };

    const sizeClasses: Record<string, string> = {
        sm: 'text-xs px-2 py-0.5',
        md: 'text-sm px-2.5 py-1',
        lg: 'text-base px-3 py-1.5',
    };

    let tierClass = $derived(tierColors[tier]);
    let statusClass = $derived(status ? getStatusColor(status) : '');
    let sizeClass = $derived(sizeClasses[size]);
</script>

<div class="inline-flex items-center gap-2">
    <span class="rounded-full font-medium {tierClass} {sizeClass}">
        {tierNames[tier]}
    </span>
    {#if status && status !== 'active'}
        <span class="rounded-full font-medium {statusClass} {sizeClass}">
            {formatStatus(status)}
        </span>
    {/if}
</div>
