interface SparklineProps {
    values: number[];
    
    // Nilai yang dianggap 100% tinggi grafik.
    max?: number;

    // Jumlah titik penuh; data lebih sedikit untuk dirapatkan ke kanan
    length?: number;
}

const WIDTH = 100;
const HEIGHT = 32;

// Grafik garis mini tanpa menggunakan dependensi (SVG)
export function Sparkline({ values, max = 100, length = 40 }: SparklineProps) {
  if (values.length < 2) {
    return <div className="sparkline sparkline-empty" aria-hidden="true" />;
  }

  const step = WIDTH / (length - 1);
  const offset = (length - values.length) * step;

  const points = values.map((value, index) => {
    const x = offset + index * step;
    const y = HEIGHT - (Math.min(max, Math.max(0, value)) / max) * HEIGHT;
    return `${x.toFixed(2)},${y.toFixed(2)}`;
  });

  const lastX = offset + (values.length - 1) * step;
  const area = `${offset.toFixed(2)},${HEIGHT} ${points.join(" ")} ${lastX.toFixed(2)},${HEIGHT}`;

  return (
    <svg
      className="sparkline"
      viewBox={`0 0 ${WIDTH} ${HEIGHT}`}
      preserveAspectRatio="none"
      aria-hidden="true"
    >
      <polygon className="sparkline-area" points={area} />
      <polyline className="sparkline-line" points={points.join(" ")} />
    </svg>
  );
}