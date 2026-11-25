import type { ComponentType } from 'react';
import { useTranslation } from '@/i18n';

/**
 * Dynamic SVG diagrams embedded in help markdown.
 *
 * A document references a diagram with an image tag whose "url" is the
 * diagram name — `![caption](diagram:relay)` — and the markdown renderer
 * (HelpSection) swaps it for the registered component. Unlike the static
 * screenshots in `images/`, these are live SVG: theme-aware colors and
 * simple SMIL animations that work in every supported browser.
 */

/** Marker colors of the shared-server nodes — one per server, echoing the
 *  colored affinity stripe the desktop client paints on shared-server tabs. */
const SHARED_SERVER_STRIPES = [
  'var(--color-success)',
  'var(--color-info)',
  'var(--color-warning)',
] as const;

function DiagramNode({
  x,
  y,
  w,
  h,
  label,
  stripe,
  emphasized,
}: {
  x: number;
  y: number;
  w: number;
  h: number;
  label: string;
  stripe?: string;
  emphasized?: boolean;
}) {
  return (
    <g>
      <rect
        x={x}
        y={y}
        width={w}
        height={h}
        rx={8}
        className={
          emphasized
            ? 'fill-golden-cream stroke-amber-gold'
            : 'fill-golden-cream stroke-golden-sand'
        }
        strokeWidth={emphasized ? 1.5 : 1}
      />
      {stripe && (
        <path
          // Left affinity stripe (8px), following the node's rounded corners.
          d={`M ${x} ${y + 8} a 8 8 0 0 1 8 -8 l 0 ${h} a 8 8 0 0 1 -8 -8 z`}
          fill={stripe}
        />
      )}
      <text
        x={x + w / 2}
        y={y + h / 2 + 4}
        textAnchor="middle"
        fontSize={12}
        className="fill-warm-black font-medium"
      >
        {label}
      </text>
    </g>
  );
}

/** A connection line with flowing dashes plus a dot travelling along it. */
function DiagramEdge({ path, delay = 0 }: { path: string; delay?: number }) {
  return (
    <g>
      <path
        d={path}
        fill="none"
        strokeWidth={1.5}
        strokeDasharray="5 5"
        className="stroke-warm-taupe"
      >
        <animate
          attributeName="stroke-dashoffset"
          from="0"
          to="-20"
          dur="1.4s"
          begin="0s"
          repeatCount="indefinite"
        />
      </path>
      <circle r={3} className="fill-amber-gold">
        <animateMotion dur="2.4s" begin={`${delay}s`} repeatCount="indefinite" path={path} />
      </circle>
    </g>
  );
}

export function RelayArchitectureDiagram() {
  const { t } = useTranslation();
  const clientLabel = t('help.relayDiagramClients');
  const relayLabel = t('help.relayDiagramRelay');
  const sharedLabel = t('help.relayDiagramShared');

  return (
    <svg
      viewBox="0 0 680 210"
      role="img"
      aria-label={t('help.relayDiagramTitle')}
      className="block w-full max-w-full rounded-standard border border-golden-sand bg-pale-sand/40"
    >
      {/* Client → relay (the single connection every client dials). */}
      <DiagramEdge path="M 156 105 H 274" />
      {/* Relay → each shared server (upstream connections). */}
      <DiagramEdge path="M 406 105 C 460 105 470 35 524 35" delay={0.4} />
      <DiagramEdge path="M 406 105 H 524" delay={1.2} />
      <DiagramEdge path="M 406 105 C 460 105 470 175 524 175" delay={0.8} />

      <DiagramNode x={24} y={75} w={132} h={60} label={clientLabel} />
      <DiagramNode x={274} y={65} w={132} h={80} label={relayLabel} emphasized />
      <DiagramNode x={524} y={15} w={132} h={40} label={sharedLabel} stripe={SHARED_SERVER_STRIPES[0]} />
      <DiagramNode x={524} y={85} w={132} h={40} label={sharedLabel} stripe={SHARED_SERVER_STRIPES[1]} />
      <DiagramNode x={524} y={155} w={132} h={40} label={sharedLabel} stripe={SHARED_SERVER_STRIPES[2]} />
    </svg>
  );
}

/** A quiet static relation (no flow animation) with an optional arrow + label. */
function DiagramStaticEdge({
  path,
  label,
  labelX,
  labelY,
  labelAnchor = 'middle',
}: {
  path: string;
  label?: string;
  labelX?: number;
  labelY?: number;
  labelAnchor?: 'start' | 'middle' | 'end';
}) {
  return (
    <g>
      <path
        d={path}
        fill="none"
        strokeWidth={1.5}
        strokeDasharray="3 4"
        className="stroke-warm-khaki"
        markerEnd="url(#ddns-edge-arrow)"
      />
      {label && labelX != null && labelY != null && (
        <text x={labelX} y={labelY} textAnchor={labelAnchor} fontSize={10} className="fill-warm-khaki">
          {label}
        </text>
      )}
    </g>
  );
}

export function DdnsArchitectureDiagram() {
  const { t } = useTranslation();

  return (
    <svg
      viewBox="0 0 680 250"
      role="img"
      aria-label={t('help.ddnsDiagramTitle')}
      className="block w-full max-w-full rounded-standard border border-golden-sand bg-pale-sand/40"
    >
      <defs>
        <marker
          id="ddns-edge-arrow"
          viewBox="0 0 8 8"
          refX="7"
          refY="4"
          markerWidth="6"
          markerHeight="6"
          orient="auto-start-reverse"
        >
          <path d="M0,0 L8,4 L0,8 z" className="fill-warm-khaki" />
        </marker>
      </defs>

      {/* The server-side pipeline lives inside one dashed group box. */}
      <rect
        x={16}
        y={34}
        width={432}
        height={152}
        rx={10}
        strokeDasharray="6 4"
        className="fill-pale-sand/30 stroke-golden-sand"
      />
      <text x={30} y={58} fontSize={11} className="fill-deep-olive font-medium">
        {t('help.ddnsDiagramServer')}
      </text>

      {/* Pipeline: manager → IP detection → addon, refreshed every cycle. */}
      <DiagramEdge path="M 152 115 H 172" delay={0.2} />
      <DiagramEdge path="M 288 115 H 308" delay={1.0} />
      <DiagramNode x={36} y={88} w={116} h={54} label={t('help.ddnsDiagramManager')} emphasized />
      <DiagramNode x={172} y={88} w={116} h={54} label={t('help.ddnsDiagramDetect')} />
      <DiagramNode x={308} y={88} w={124} h={54} label={t('help.ddnsDiagramAddon')} />

      {/* The addon updates the provider's DNS record over HTTPS. */}
      <DiagramEdge path="M 432 115 C 478 115 486 64 524 64" delay={0.6} />
      <text x={478} y={46} textAnchor="middle" fontSize={10} className="fill-warm-khaki">
        {t('help.ddnsDiagramUpdate')}
      </text>
      <DiagramNode x={524} y={38} w={136} h={52} label={t('help.ddnsDiagramProvider')} />

      {/* Clients resolve the domain and connect by name. */}
      <DiagramStaticEdge path="M 592 168 V 90" label={t('help.ddnsDiagramResolve')} labelX={600} labelY={132} labelAnchor="start" />
      <DiagramNode x={524} y={168} w={136} h={52} label={t('help.relayDiagramClients')} />
      <DiagramStaticEdge path="M 524 194 C 484 194 480 168 448 152" label={t('help.ddnsDiagramConnect')} labelX={486} labelY={216} />
    </svg>
  );
}

/** Diagram registry — markdown `diagram/<name>` references resolve here. */
const helpDiagrams: Record<string, ComponentType> = {
  relay: RelayArchitectureDiagram,
  ddns: DdnsArchitectureDiagram,
};

export function getHelpDiagram(name: string): ComponentType | undefined {
  return helpDiagrams[name];
}
