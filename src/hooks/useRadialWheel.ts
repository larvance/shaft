import { useMemo, useState, useCallback } from "react";
import { SectorGeometry, WheelItem } from "../types/wheel";

interface RadialWheelConfig {
  items: WheelItem[];
  cx?: number;
  cy?: number;
  innerRadius?: number;
  outerRadius?: number;
  hoverRadiusDelta?: number;
  gapRadians?: number;
}

export function useRadialWheel({
  items,
  cx = 200,
  cy = 200,
  innerRadius = 70,
  outerRadius = 160,
  hoverRadiusDelta = 16,
  gapRadians = 0.035,
}: RadialWheelConfig) {
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);

  const n = items.length;
  const deltaTheta = n > 0 ? (2 * Math.PI) / n : 0;
  const baseAngle = -Math.PI / 2; // 12 o'clock

  const sectors: SectorGeometry[] = useMemo(() => {
    if (n === 0) return [];

    return items.map((item, i) => {
      const isHovered = hoveredIndex === i;
      const currentOuterRadius = isHovered
        ? outerRadius + hoverRadiusDelta
        : outerRadius;

      const startAngle = baseAngle + i * deltaTheta + gapRadians / 2;
      const endAngle = baseAngle + (i + 1) * deltaTheta - gapRadians / 2;
      const midAngle = (startAngle + endAngle) / 2;

      const p1 = [
        cx + innerRadius * Math.cos(startAngle),
        cy + innerRadius * Math.sin(startAngle),
      ];
      const p2 = [
        cx + currentOuterRadius * Math.cos(startAngle),
        cy + currentOuterRadius * Math.sin(startAngle),
      ];
      const p3 = [
        cx + currentOuterRadius * Math.cos(endAngle),
        cy + currentOuterRadius * Math.sin(endAngle),
      ];
      const p4 = [
        cx + innerRadius * Math.cos(endAngle),
        cy + innerRadius * Math.sin(endAngle),
      ];

      const largeArc = endAngle - startAngle > Math.PI ? 1 : 0;

      const pathData = [
        `M ${p1[0].toFixed(2)} ${p1[1].toFixed(2)}`,
        `L ${p2[0].toFixed(2)} ${p2[1].toFixed(2)}`,
        `A ${currentOuterRadius.toFixed(2)} ${currentOuterRadius.toFixed(2)} 0 ${largeArc} 1 ${p3[0].toFixed(2)} ${p3[1].toFixed(2)}`,
        `L ${p4[0].toFixed(2)} ${p4[1].toFixed(2)}`,
        `A ${innerRadius.toFixed(2)} ${innerRadius.toFixed(2)} 0 ${largeArc} 0 ${p1[0].toFixed(2)} ${p1[1].toFixed(2)}`,
        `Z`,
      ].join(" ");

      const midRadius = (innerRadius + currentOuterRadius) / 2;
      const iconPosition = {
        x: cx + midRadius * Math.cos(midAngle),
        y: cy + midRadius * Math.sin(midAngle),
      };

      const labelRadius = currentOuterRadius - 18;
      const labelPosition = {
        x: cx + labelRadius * Math.cos(midAngle),
        y: cy + labelRadius * Math.sin(midAngle),
      };

      return {
        index: i,
        item,
        pathData,
        startAngle,
        endAngle,
        midAngle,
        iconPosition,
        labelPosition,
        isHovered,
      };
    });
  }, [items, n, deltaTheta, baseAngle, gapRadians, cx, cy, innerRadius, outerRadius, hoverRadiusDelta, hoveredIndex]);

  const handlePointerMove = useCallback(
    (clientX: number, clientY: number, svgRect: DOMRect) => {
      if (n === 0) {
        setHoveredIndex(null);
        return;
      }

      // Coordinates relative to SVG center (in 400x400 viewBox space)
      const scaleX = 400 / svgRect.width;
      const scaleY = 400 / svgRect.height;
      const mouseX = (clientX - svgRect.left) * scaleX;
      const mouseY = (clientY - svgRect.top) * scaleY;

      const dx = mouseX - cx;
      const dy = mouseY - cy;
      const distance = Math.sqrt(dx * dx + dy * dy);

      // Deadzone or outside interaction circle
      if (distance < innerRadius - 12 || distance > outerRadius + 60) {
        setHoveredIndex(null);
        return;
      }

      let angle = Math.atan2(dy, dx);
      let normalized = (angle - baseAngle) % (2 * Math.PI);
      if (normalized < 0) {
        normalized += 2 * Math.PI;
      }

      const index = Math.floor(normalized / deltaTheta);
      if (index >= 0 && index < n) {
        setHoveredIndex(index);
      } else {
        setHoveredIndex(null);
      }
    },
    [n, cx, cy, innerRadius, outerRadius, deltaTheta, baseAngle]
  );

  const handlePointerLeave = useCallback(() => {
    setHoveredIndex(null);
  }, []);

  return {
    sectors,
    hoveredIndex,
    setHoveredIndex,
    handlePointerMove,
    handlePointerLeave,
  };
}
