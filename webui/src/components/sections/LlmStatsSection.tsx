import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Activity,
  AlertTriangle,
  BarChart3,
  Clock,
  Cpu,
  Loader2,
  RefreshCw,
  Sparkles,
  Zap,
} from 'lucide-react';
import { getLlmStats, type LlmStatsParams } from '@/api/llm';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { SelectField } from '@/components/shared/SelectField';
import { useTranslation } from '@/i18n';
import type { LlmStatsResponse } from '@/types';

// ── Color palette ──────────────────────────────────────────────────
const CHART_COLORS = [
  '#43d56f',
  '#3b9eff',
  '#a855f7',
  '#f59e0b',
  '#ef4444',
  '#06b6d4',
  '#ec4899',
  '#84cc16',
];

function chartColor(i: number): string {
  return CHART_COLORS[i % CHART_COLORS.length];
}

// ── Helpers ────────────────────────────────────────────────────────
function fmtNum(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`;
  return String(Math.round(n));
}

function fmtMs(ms: number): string {
  if (ms >= 1000) return `${(ms / 1000).toFixed(1)}s`;
  return `${Math.round(ms)}ms`;
}

function fmtPercent(count: number, total: number): string {
  if (total === 0) return '0%';
  return `${((count / total) * 100).toFixed(1)}%`;
}

// ── Time range options ─────────────────────────────────────────────
const TIME_RANGES = [
  { key: '24h', label: '24h', ms: 24 * 3600_000 },
  { key: '7d', label: '7d', ms: 7 * 24 * 3600_000 },
  { key: '30d', label: '30d', ms: 30 * 24 * 3600_000 },
  { key: '90d', label: '90d', ms: 90 * 24 * 3600_000 },
  { key: 'all', label: 'All', ms: 0 },
];

// ── Glow card wrapper ──────────────────────────────────────────────
function GlowCard({
  children,
  className = '',
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <div
      className={`relative rounded-lg border border-[var(--color-border)] overflow-hidden ${className}`}
      style={{
        background:
          'linear-gradient(135deg, var(--color-surface) 0%, var(--color-surface-2) 100%)',
      }}
    >
      {children}
    </div>
  );
}

// ── Animated counter ───────────────────────────────────────────────
function StatValue({ value, suffix = '' }: { value: string; suffix?: string }) {
  return (
    <span
      className="text-2xl font-semibold tracking-tight"
      style={{ color: 'var(--color-text)' }}
    >
      {value}
      {suffix && (
        <span className="text-sm font-normal ml-0.5" style={{ color: 'var(--color-text-subtle)' }}>
          {suffix}
        </span>
      )}
    </span>
  );
}

// ── Summary cards ──────────────────────────────────────────────────
function SummaryCards({ data }: { data: LlmStatsResponse }) {
  const { summary } = data;
  const successRate = fmtPercent(summary.successCount, summary.totalRequests);

  const cards = [
    {
      icon: <Zap className="w-4 h-4" />,
      label: 'Total Requests',
      value: fmtNum(summary.totalRequests),
      accent: '#43d56f',
    },
    {
      icon: <Cpu className="w-4 h-4" />,
      label: 'Total Tokens',
      value: fmtNum(summary.totalTokens),
      accent: '#3b9eff',
    },
    {
      icon: <Activity className="w-4 h-4" />,
      label: 'Success Rate',
      value: successRate.replace('%', ''),
      suffix: '%',
      accent: '#a855f7',
    },
    {
      icon: <Clock className="w-4 h-4" />,
      label: 'Avg Latency',
      value: fmtMs(summary.avgDurationMs).replace(/ms|s/, ''),
      suffix: fmtMs(summary.avgDurationMs).includes('s') ? 's' : 'ms',
      accent: '#f59e0b',
    },
  ];

  return (
    <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
      {cards.map((card) => (
        <GlowCard key={card.label} className="p-4">
          <div className="flex items-center gap-2 mb-2">
            <div
              className="w-7 h-7 rounded-md flex items-center justify-center"
              style={{ backgroundColor: `${card.accent}20`, color: card.accent }}
            >
              {card.icon}
            </div>
            <span
              className="text-xs font-medium uppercase tracking-wider"
              style={{ color: 'var(--color-text-subtle)' }}
            >
              {card.label}
            </span>
          </div>
          <StatValue value={card.value} suffix={card.suffix} />
        </GlowCard>
      ))}
    </div>
  );
}

// ── Donut chart (SVG) ──────────────────────────────────────────────
function DonutChart({
  segments,
  size = 140,
  strokeWidth = 22,
}: {
  segments: { value: number; color: string; label: string }[];
  size?: number;
  strokeWidth?: number;
}) {
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const total = segments.reduce((s, seg) => s + seg.value, 0);

  let cumulative = 0;
  const arcs = segments.map((seg) => {
    const fraction = total > 0 ? seg.value / total : 0;
    const offset = circumference * (1 - cumulative);
    cumulative += fraction;
    return {
      ...seg,
      fraction,
      offset,
      dashLen: circumference * fraction,
    };
  });

  return (
    <div className="flex items-center gap-4">
      <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`}>
        {/* background ring */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="var(--color-border)"
          strokeWidth={strokeWidth}
        />
        {/* segments */}
        {arcs.map((arc, i) =>
          arc.fraction > 0 ? (
            <circle
              key={i}
              cx={size / 2}
              cy={size / 2}
              r={radius}
              fill="none"
              stroke={arc.color}
              strokeWidth={strokeWidth}
              strokeDasharray={`${arc.dashLen} ${circumference - arc.dashLen}`}
              strokeDashoffset={arc.offset}
              strokeLinecap="round"
              style={{ transition: 'stroke-dasharray 0.6s ease, stroke-dashoffset 0.6s ease' }}
            />
          ) : null,
        )}
      </svg>
      <div className="flex flex-col gap-1 text-xs">
        {segments.map((seg, i) => (
          <div key={i} className="flex items-center gap-2">
            <span
              className="w-2.5 h-2.5 rounded-sm shrink-0"
              style={{ backgroundColor: seg.color }}
            />
            <span style={{ color: 'var(--color-text-muted)' }}>
              {seg.label}
            </span>
            <span className="ml-auto font-medium" style={{ color: 'var(--color-text)' }}>
              {fmtPercent(seg.value, total)}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

// ── Bar chart (SVG) ────────────────────────────────────────────────
function BarChart({
  items,
  maxBars = 8,
}: {
  items: { label: string; value: number; color: string }[];
  maxBars?: number;
}) {
  const data = items.slice(0, maxBars);
  const maxVal = Math.max(...data.map((d) => d.value), 1);

  const padL = 36;
  const padR = 8;
  const chartW = 400;
  const chartH = 180;
  const barPad = 8;
  const innerW = chartW - padL - padR;
  const barW = Math.max(12, (innerW - barPad * (data.length + 1)) / data.length);
  const labelH = 32;
  const totalH = chartH + labelH;

  return (
    <svg
      viewBox={`0 0 ${chartW} ${totalH}`}
      className="w-full"
      preserveAspectRatio="xMidYMid meet"
    >
      {/* grid lines */}
      {[0, 0.25, 0.5, 0.75, 1].map((frac) => {
        const y = chartH * (1 - frac);
        return (
          <g key={frac}>
            <line
              x1={padL}
              y1={y}
              x2={chartW - padR}
              y2={y}
              stroke="var(--color-border)"
              strokeWidth={0.5}
              strokeDasharray="3,3"
            />
            <text
              x={padL - 4}
              y={y + 3}
              textAnchor="end"
              fill="var(--color-text-subtle)"
              fontSize={8}
              fontFamily="Inter, system-ui, sans-serif"
            >
              {fmtNum(maxVal * frac)}
            </text>
          </g>
        );
      })}

      {/* bars */}
      {data.map((item, i) => {
        const x = padL + barPad + i * (barW + barPad);
        const barH = (item.value / maxVal) * (chartH - 4);
        const y = chartH - barH;

        return (
          <g key={i}>
            <defs>
              <linearGradient id={`barGrad-${i}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor={item.color} stopOpacity={0.9} />
                <stop offset="100%" stopColor={item.color} stopOpacity={0.5} />
              </linearGradient>
            </defs>
            <rect
              x={x}
              y={y}
              width={barW}
              height={barH}
              rx={3}
              fill={`url(#barGrad-${i})`}
              style={{ transition: 'height 0.5s ease, y 0.5s ease' }}
            />
            {/* value label on top */}
            <text
              x={x + barW / 2}
              y={y - 4}
              textAnchor="middle"
              fill="var(--color-text-muted)"
              fontSize={8}
              fontFamily="Inter, system-ui, sans-serif"
            >
              {fmtNum(item.value)}
            </text>
            {/* bottom label */}
            <text
              x={x + barW / 2}
              y={chartH + 14}
              textAnchor="middle"
              fill="var(--color-text-subtle)"
              fontSize={8}
              fontFamily="Inter, system-ui, sans-serif"
            >
              {item.label.length > 10 ? item.label.slice(0, 9) + '...' : item.label}
            </text>
          </g>
        );
      })}
    </svg>
  );
}

// ── Area chart (SVG) ───────────────────────────────────────────────
function AreaChart({ data }: { data: { label: string; value: number }[] }) {
  if (data.length === 0) return <EmptyChart />;

  const chartW = 400;
  const chartH = 160;
  const padL = 36;
  const padR = 8;
  const padY = 16;
  const maxVal = Math.max(...data.map((d) => d.value), 1);
  const innerW = chartW - padL - padR;

  const labelH = 28;
  const totalH = chartH + labelH;

  // Single data point — show a horizontal bar instead of a dot
  if (data.length === 1) {
    const y = padY + (1 - data[0].value / maxVal) * (chartH - padY * 2);
    return (
      <svg
        viewBox={`0 0 ${chartW} ${totalH}`}
        className="w-full"
        preserveAspectRatio="xMidYMid meet"
      >
        <defs>
          <linearGradient id="singleBarGrad" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#43d56f" stopOpacity={0.35} />
            <stop offset="100%" stopColor="#43d56f" stopOpacity={0.02} />
          </linearGradient>
        </defs>

        {/* Y-axis label */}
        <text
          x={padL - 4}
          y={y + 3}
          textAnchor="end"
          fill="var(--color-text-subtle)"
          fontSize={8}
          fontFamily="Inter, system-ui, sans-serif"
        >
          {fmtNum(data[0].value)}
        </text>

        {/* filled area band */}
        <rect x={padL} y={y} width={innerW} height={chartH - padY - y} fill="url(#singleBarGrad)" rx={4} />

        {/* value line */}
        <line
          x1={padL}
          y1={y}
          x2={chartW - padR}
          y2={y}
          stroke="#43d56f"
          strokeWidth={2}
          strokeDasharray="6,4"
          strokeLinecap="round"
        />

        {/* dot at center */}
        <circle cx={padL + innerW / 2} cy={y} r={4} fill="#43d56f" stroke="var(--color-surface)" strokeWidth={2} />

        {/* x-axis label */}
        <text
          x={chartW / 2}
          y={chartH + 16}
          textAnchor="middle"
          fill="var(--color-text-subtle)"
          fontSize={7.5}
          fontFamily="Inter, system-ui, sans-serif"
        >
          {data[0].label.length > 10 ? data[0].label.slice(5) : data[0].label}
        </text>
      </svg>
    );
  }

  const points = data.map((d, i) => ({
    x: padL + (i / Math.max(data.length - 1, 1)) * innerW,
    y: padY + (1 - d.value / maxVal) * (chartH - padY * 2),
  }));

  const pathD = points
    .map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`)
    .join(' ');

  const areaD = `${pathD} L ${points[points.length - 1].x} ${chartH - padY} L ${points[0].x} ${chartH - padY} Z`;

  // Show at most 6 labels on x-axis
  const labelStep = Math.max(1, Math.floor(data.length / 6));

  return (
    <svg
      viewBox={`0 0 ${chartW} ${totalH}`}
      className="w-full"
      preserveAspectRatio="xMidYMid meet"
    >
      <defs>
        <linearGradient id="areaGrad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stopColor="#43d56f" stopOpacity={0.35} />
          <stop offset="100%" stopColor="#43d56f" stopOpacity={0.02} />
        </linearGradient>
      </defs>

      {/* Y-axis labels */}
      {[0, 0.5, 1].map((frac) => {
        const y = padY + (1 - frac) * (chartH - padY * 2);
        return (
          <g key={frac}>
            <line
              x1={padL}
              y1={y}
              x2={chartW - padR}
              y2={y}
              stroke="var(--color-border)"
              strokeWidth={0.5}
              strokeDasharray="3,3"
            />
            <text
              x={padL - 4}
              y={y + 3}
              textAnchor="end"
              fill="var(--color-text-subtle)"
              fontSize={8}
              fontFamily="Inter, system-ui, sans-serif"
            >
              {fmtNum(maxVal * frac)}
            </text>
          </g>
        );
      })}

      {/* area fill */}
      <path d={areaD} fill="url(#areaGrad)" />

      {/* line */}
      <path
        d={pathD}
        fill="none"
        stroke="#43d56f"
        strokeWidth={2}
        strokeLinejoin="round"
        strokeLinecap="round"
      />

      {/* dots */}
      {points.map((p, i) => (
        <circle
          key={i}
          cx={p.x}
          cy={p.y}
          r={2.5}
          fill="#43d56f"
          stroke="var(--color-surface)"
          strokeWidth={1.5}
        />
      ))}

      {/* x-axis labels */}
      {data.map(
        (d, i) =>
          i % labelStep === 0 && (
            <text
              key={i}
              x={points[i].x}
              y={chartH + 16}
              textAnchor="middle"
              fill="var(--color-text-subtle)"
              fontSize={7.5}
              fontFamily="Inter, system-ui, sans-serif"
            >
              {d.label.length > 10 ? d.label.slice(5) : d.label}
            </text>
          ),
      )}
    </svg>
  );
}

function EmptyChart() {
  return (
    <div className="flex items-center justify-center h-40" style={{ color: 'var(--color-text-subtle)' }}>
      <div className="text-center">
        <BarChart3 className="w-8 h-8 mx-auto mb-2 opacity-30" />
        <p className="text-xs">No data available</p>
      </div>
    </div>
  );
}

// ── Tab bar ────────────────────────────────────────────────────────
function TabBar({
  tabs,
  active,
  onChange,
}: {
  tabs: { key: string; label: string }[];
  active: string;
  onChange: (key: string) => void;
}) {
  return (
    <div className="flex gap-1 p-0.5 rounded-lg" style={{ background: 'var(--color-bg-inset)' }}>
      {tabs.map((tab) => (
        <button
          key={tab.key}
          onClick={() => onChange(tab.key)}
          className="px-3 py-1.5 text-xs font-medium rounded-md transition-all duration-200"
          style={{
            background: active === tab.key ? 'var(--color-surface)' : 'transparent',
            color: active === tab.key ? 'var(--color-text)' : 'var(--color-text-subtle)',
            boxShadow: active === tab.key ? '0 1px 3px rgba(0,0,0,0.1)' : 'none',
          }}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}

// ── Main component ─────────────────────────────────────────────────
export function LlmStatsSection() {
  const { t } = useTranslation();
  const [data, setData] = useState<LlmStatsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [timeRange, setTimeRange] = useState('7d');
  const [categoryFilter, setCategoryFilter] = useState('');
  const [addonFilter, setAddonFilter] = useState('');
  const [chartTab, setChartTab] = useState<'category' | 'addon' | 'model'>('category');

  const buildParams = useCallback((): LlmStatsParams => {
    const now = Date.now();
    const range = TIME_RANGES.find((r) => r.key === timeRange);
    const params: LlmStatsParams = {};
    if (range && range.ms > 0) {
      params.fromMs = now - range.ms;
      params.toMs = now;
    }
    if (categoryFilter) params.category = categoryFilter;
    if (addonFilter) params.addon = addonFilter;
    return params;
  }, [timeRange, categoryFilter, addonFilter]);

  const fetchStats = useCallback(async () => {
    setLoading(true);
    try {
      const result = await getLlmStats(buildParams());
      setData(result);
    } catch {
      // ignore
    } finally {
      setLoading(false);
    }
  }, [buildParams]);

  useEffect(() => {
    fetchStats();
  }, [fetchStats]);

  // Extract unique categories and addons for filter dropdowns
  const categories = useMemo(
    () => (data ? [...new Set(data.byCategory.map((c) => c.category))] : []),
    [data],
  );
  const addons = useMemo(
    () => (data ? [...new Set(data.byAddon.map((a) => a.addonName))] : []),
    [data],
  );

  // Build chart data based on selected tab
  const donutSegments = useMemo(() => {
    if (!data) return [];
    const source =
      chartTab === 'category'
        ? data.byCategory.map((c) => ({ tokens: c.totalTokens, label: c.category }))
        : chartTab === 'addon'
          ? data.byAddon.map((a) => ({ tokens: a.totalTokens, label: a.addonName }))
          : data.byModel.map((m) => ({ tokens: m.totalTokens, label: m.modelName }));
    return source.map((item, i) => ({
      value: item.tokens,
      color: chartColor(i),
      label: item.label,
    }));
  }, [data, chartTab]);

  const barData = useMemo(() => {
    if (!data) return [];
    const source =
      chartTab === 'category'
        ? data.byCategory.map((c) => ({ tokens: c.totalTokens, label: c.category }))
        : chartTab === 'addon'
          ? data.byAddon.map((a) => ({ tokens: a.totalTokens, label: a.addonName }))
          : data.byModel.map((m) => ({ tokens: m.totalTokens, label: m.modelName }));
    return source.map((item, i) => ({
      label: item.label,
      value: item.tokens,
      color: chartColor(i),
    }));
  }, [data, chartTab]);

  // Unified detail rows based on chartTab
  const detailRows = useMemo(() => {
    if (!data) return [];
    if (chartTab === 'category')
      return data.byCategory.map((c) => ({ name: c.category, ...c }));
    if (chartTab === 'addon')
      return data.byAddon.map((a) => ({ name: a.addonName, ...a }));
    return data.byModel.map((m) => ({ name: m.modelName, ...m }));
  }, [data, chartTab]);

  const detailLabel = useMemo(
    () => (chartTab === 'category' ? 'Category' : chartTab === 'addon' ? 'Addon' : 'Model'),
    [chartTab],
  );

  // Input/Output ratio per item in selected dimension
  const ioRatioItems = useMemo(() => {
    if (!data) return [];
    const src =
      chartTab === 'category'
        ? data.byCategory
        : chartTab === 'addon'
          ? data.byAddon
          : data.byModel;
    return src.map((item, i) => {
      const input = 'inputTokens' in item ? item.inputTokens : 0;
      const output = 'outputTokens' in item ? item.outputTokens : 0;
      const total = input + output || 1;
      return {
        label: chartTab === 'category' ? (item as { category: string }).category : chartTab === 'addon' ? (item as { addonName: string }).addonName : (item as { modelName: string }).modelName,
        color: chartColor(i),
        input,
        output,
        total,
      };
    });
  }, [data, chartTab]);

  const timeSeriesData = useMemo(() => {
    if (!data) return [];
    return data.timeSeries.map((ts) => ({
      label: ts.bucket,
      value: ts.totalTokens,
    }));
  }, [data]);

  if (loading && !data) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-6 h-6 animate-spin" style={{ color: 'var(--color-text)' }} />
      </div>
    );
  }

  if (!data) {
    return (
      <div className="flex items-center gap-2 py-8" style={{ color: 'var(--color-text)' }}>
        <AlertTriangle className="w-5 h-5" />
        <span className="font-light">{t('llmStats.loadError')}</span>
      </div>
    );
  }

  const hasData = data.summary.totalRequests > 0;

  return (
    <div className="space-y-5">
      <SectionHeader title={t('llmStats.title')} description={t('llmStats.description')}>
        <button
          onClick={fetchStats}
          className="btn-secondary btn-pill px-3 py-1.5 text-xs flex items-center gap-1.5"
          disabled={loading}
        >
          <RefreshCw className={`w-3 h-3 ${loading ? 'animate-spin' : ''}`} />
          {t('common.refresh')}
        </button>
      </SectionHeader>

      {/* Filters */}
      <div className="flex flex-wrap items-center gap-3">
        {/* Time range pills */}
        <div className="flex gap-1 p-0.5 rounded-lg" style={{ background: 'var(--color-bg-inset)' }}>
          {TIME_RANGES.map((range) => (
            <button
              key={range.key}
              onClick={() => setTimeRange(range.key)}
              className="px-2.5 py-1 text-xs font-medium rounded-md transition-all duration-200"
              style={{
                background: timeRange === range.key ? 'var(--color-surface)' : 'transparent',
                color: timeRange === range.key ? 'var(--color-text)' : 'var(--color-text-subtle)',
                boxShadow: timeRange === range.key ? '0 1px 3px rgba(0,0,0,0.08)' : 'none',
              }}
            >
              {range.label}
            </button>
          ))}
        </div>

        {/* Category filter */}
        {categories.length > 1 && (
          <div className="w-40">
            <SelectField
              value={categoryFilter}
              onChange={(e) => setCategoryFilter(e.target.value)}
              ariaLabel={t('llmStats.allCategories')}
              options={[
                { value: '', label: t('llmStats.allCategories') },
                ...categories.map((cat) => ({ value: cat, label: cat })),
              ]}
            />
          </div>
        )}

        {/* Addon filter */}
        {addons.length > 1 && (
          <div className="w-40">
            <SelectField
              value={addonFilter}
              onChange={(e) => setAddonFilter(e.target.value)}
              ariaLabel={t('llmStats.allAddons')}
              options={[
                { value: '', label: t('llmStats.allAddons') },
                ...addons.map((ad) => ({ value: ad, label: ad })),
              ]}
            />
          </div>
        )}
      </div>

      {!hasData ? (
        <GlowCard className="p-8 text-center">
          <Sparkles className="w-8 h-8 mx-auto mb-3 opacity-30" style={{ color: 'var(--color-accent)' }} />
          <p className="text-sm font-medium" style={{ color: 'var(--color-text-muted)' }}>
            {t('llmStats.noData')}
          </p>
          <p className="text-xs mt-1" style={{ color: 'var(--color-text-subtle)' }}>
            {t('llmStats.noDataDesc')}
          </p>
        </GlowCard>
      ) : (
        <>
          {/* Summary cards */}
          <SummaryCards data={data} />

          {/* Two-column layout: donut + bar */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-3">
            {/* Token distribution (donut) */}
            <GlowCard className="p-4">
              <div className="flex items-center justify-between mb-4">
                <h3
                  className="text-sm font-medium"
                  style={{ color: 'var(--color-text)' }}
                >
                  {t('llmStats.tokenDistribution')}
                </h3>
                <TabBar
                  tabs={[
                    { key: 'category', label: t('llmStats.byCategory') },
                    { key: 'addon', label: t('llmStats.byAddon') },
                    { key: 'model', label: t('llmStats.byModel') },
                  ]}
                  active={chartTab}
                  onChange={(k) => setChartTab(k as typeof chartTab)}
                />
              </div>
              {donutSegments.length > 0 ? (
                <DonutChart segments={donutSegments} />
              ) : (
                <EmptyChart />
              )}
            </GlowCard>

            {/* Token bar chart */}
            <GlowCard className="p-4">
              <h3
                className="text-sm font-medium mb-4"
                style={{ color: 'var(--color-text)' }}
              >
                {t('llmStats.tokenUsage')}
              </h3>
              {barData.length > 0 ? <BarChart items={barData} /> : <EmptyChart />}
            </GlowCard>
          </div>

          {/* Time series area chart */}
          <GlowCard className="p-4">
            <h3
              className="text-sm font-medium mb-4"
              style={{ color: 'var(--color-text)' }}
            >
              {t('llmStats.usageOverTime')}
            </h3>
            {timeSeriesData.length > 0 ? (
              <AreaChart data={timeSeriesData} />
            ) : (
              <EmptyChart />
            )}
          </GlowCard>

          {/* Detail table */}
          {detailRows.length > 0 && (
            <GlowCard>
              <div className="p-4 border-b" style={{ borderColor: 'var(--color-border)' }}>
                <h3
                  className="text-sm font-medium"
                  style={{ color: 'var(--color-text)' }}
                >
                  {t('llmStats.detailBy' + chartTab.charAt(0).toUpperCase() + chartTab.slice(1))}
                </h3>
              </div>
              <div className="overflow-x-auto">
                <table className="w-full text-xs">
                  <thead>
                    <tr
                      className="border-b"
                      style={{ borderColor: 'var(--color-border)' }}
                    >
                      {[detailLabel, 'Requests', 'Input', 'Output', 'Total', 'Success', 'Avg Duration'].map(
                        (h) => (
                          <th
                            key={h}
                            className="px-4 py-2 text-left font-medium uppercase tracking-wider"
                            style={{ color: 'var(--color-text-subtle)' }}
                          >
                            {h}
                          </th>
                        ),
                      )}
                    </tr>
                  </thead>
                  <tbody>
                    {detailRows.map((row, i) => (
                      <tr
                        key={i}
                        className="border-b last:border-b-0 transition-colors duration-150"
                        style={{
                          borderColor: 'var(--color-border)',
                        }}
                        onMouseEnter={(e) =>
                          (e.currentTarget.style.background = 'var(--color-bg-inset)')
                        }
                        onMouseLeave={(e) =>
                          (e.currentTarget.style.background = 'transparent')
                        }
                      >
                        <td className="px-4 py-2.5 font-medium" style={{ color: 'var(--color-text)' }}>
                          <div className="flex items-center gap-2">
                            <span
                              className="w-2 h-2 rounded-sm"
                              style={{ backgroundColor: chartColor(i) }}
                            />
                            {row.name}
                          </div>
                        </td>
                        <td className="px-4 py-2.5" style={{ color: 'var(--color-text-muted)' }}>
                          {fmtNum(row.requestCount)}
                        </td>
                        <td className="px-4 py-2.5" style={{ color: 'var(--color-text-muted)' }}>
                          {fmtNum(row.inputTokens)}
                        </td>
                        <td className="px-4 py-2.5" style={{ color: 'var(--color-text-muted)' }}>
                          {fmtNum(row.outputTokens)}
                        </td>
                        <td className="px-4 py-2.5 font-medium" style={{ color: 'var(--color-text)' }}>
                          {fmtNum(row.totalTokens)}
                        </td>
                        <td className="px-4 py-2.5">
                          <span
                            className="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium"
                            style={{
                              backgroundColor:
                                row.successCount === row.requestCount
                                  ? 'rgba(67,213,111,0.15)'
                                  : 'rgba(245,158,11,0.15)',
                              color:
                                row.successCount === row.requestCount
                                  ? '#43d56f'
                                  : '#f59e0b',
                            }}
                          >
                            {fmtPercent(row.successCount, row.requestCount)}
                          </span>
                        </td>
                        <td className="px-4 py-2.5" style={{ color: 'var(--color-text-muted)' }}>
                          {fmtMs(row.avgDurationMs)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </GlowCard>
          )}

          {/* Input/Output ratio per dimension item */}
          {ioRatioItems.length > 0 && (
            <GlowCard className="p-4">
              <h3
                className="text-sm font-medium mb-3"
                style={{ color: 'var(--color-text)' }}
              >
                {t('llmStats.inputOutputRatio')}
              </h3>
              <div className="space-y-3">
                {ioRatioItems.map((item) => (
                  <div key={item.label}>
                    <div className="flex items-center gap-2 mb-1">
                      <span
                        className="w-2 h-2 rounded-sm shrink-0"
                        style={{ backgroundColor: item.color }}
                      />
                      <span className="text-xs font-medium" style={{ color: 'var(--color-text)' }}>
                        {item.label}
                      </span>
                    </div>
                    <div
                      className="h-4 rounded-full overflow-hidden flex"
                      style={{ background: 'var(--color-bg-inset)' }}
                    >
                      <div
                        className="h-full transition-all duration-700"
                        style={{
                          width: `${(item.input / item.total) * 100}%`,
                          background: 'linear-gradient(90deg, #3b9eff, #43d56f)',
                          minWidth: item.input > 0 ? '4px' : '0',
                        }}
                      />
                      <div
                        className="h-full transition-all duration-700"
                        style={{
                          width: `${(item.output / item.total) * 100}%`,
                          background: 'linear-gradient(90deg, #a855f7, #ec4899)',
                          minWidth: item.output > 0 ? '4px' : '0',
                        }}
                      />
                    </div>
                    <div className="flex justify-between text-[10px] mt-0.5" style={{ color: 'var(--color-text-subtle)' }}>
                      <span>
                        In: {fmtNum(item.input)} ({fmtPercent(item.input, item.total)})
                      </span>
                      <span>
                        Out: {fmtNum(item.output)} ({fmtPercent(item.output, item.total)})
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </GlowCard>
          )}
        </>
      )}
    </div>
  );
}
