export type DockEdge = "top" | "left" | "right";

export interface Point {
  x: number;
  y: number;
}

export interface BoxSize {
  width: number;
  height: number;
}

export interface WorkArea extends Point, BoxSize {}

export interface DockPlacement {
  edge: DockEdge;
  position: Point;
  size: BoxSize;
}

interface DockPlacementInput {
  position: Point;
  currentSize: BoxSize;
  workArea: WorkArea;
  horizontalSize: BoxSize;
  sideSize: BoxSize;
  margin?: number;
  threshold?: number;
}

const DEFAULT_MARGIN = 8;
const DEFAULT_THRESHOLD = 80;

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function calculateTopPosition({
  position,
  currentSize,
  workArea,
  horizontalSize,
  margin,
}: {
  position: Point;
  currentSize: BoxSize;
  workArea: WorkArea;
  horizontalSize: BoxSize;
  margin: number;
}): Point {
  const rightEdge = workArea.x + workArea.width;
  const draggedCenterX = position.x + currentSize.width / 2;

  return {
    x: clamp(
      Math.round(draggedCenterX - horizontalSize.width / 2),
      workArea.x + margin,
      rightEdge - horizontalSize.width - margin,
    ),
    y: workArea.y,
  };
}

export function calculateDockPlacement({
  position,
  currentSize,
  workArea,
  horizontalSize,
  sideSize,
  margin = DEFAULT_MARGIN,
  threshold = DEFAULT_THRESHOLD,
}: DockPlacementInput): DockPlacement {
  const rightEdge = workArea.x + workArea.width;
  const bottomEdge = workArea.y + workArea.height;
  const windowRightEdge = position.x + currentSize.width;
  const distanceToTop = Math.abs(position.y - workArea.y);
  const distanceToLeft = Math.abs(position.x - workArea.x);
  const distanceToRight = Math.abs(rightEdge - windowRightEdge);

  if (distanceToTop <= threshold) {
    return {
      edge: "top",
      position: calculateTopPosition({ position, currentSize, workArea, horizontalSize, margin }),
      size: horizontalSize,
    };
  }

  if (distanceToLeft <= threshold && distanceToLeft <= distanceToTop) {
    return {
      edge: "left",
      position: {
        x: workArea.x + margin,
        y: clamp(position.y, workArea.y + margin, bottomEdge - sideSize.height - margin),
      },
      size: sideSize,
    };
  }

  if (distanceToRight <= threshold && distanceToRight <= distanceToTop) {
    return {
      edge: "right",
      position: {
        x: rightEdge - sideSize.width - margin,
        y: clamp(position.y, workArea.y + margin, bottomEdge - sideSize.height - margin),
      },
      size: sideSize,
    };
  }

  return {
    edge: "top",
    position: calculateTopPosition({ position, currentSize, workArea, horizontalSize, margin }),
    size: horizontalSize,
  };
}
