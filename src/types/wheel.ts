export type WheelMode = "format" | "tool" | "action";

export interface WheelItem {
  id: string;
  label: string;
  sublabel?: string;
  icon?: string;
  category?: string;
  payload: unknown;
}

export interface PolarCoordinate {
  r: number;
  theta: number;
}

export interface SectorGeometry {
  index: number;
  item: WheelItem;
  pathData: string;
  startAngle: number;
  endAngle: number;
  midAngle: number;
  iconPosition: { x: number; y: number };
  labelPosition: { x: number; y: number };
  isHovered: boolean;
}
