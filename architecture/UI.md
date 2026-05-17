# UI.md - User Interface Architecture (DENSE)

**Authority:** guidance (UI patterns and component architecture)
**Layer:** Guides
**Binding:** No
**Scope:** UI architecture patterns, component design, interaction models, and rendering strategies
**Non-goals:** specific framework implementations, visual design systems, or branding guidelines

---

## 1. UI Architecture Philosophy

### 1.1 Intent-Driven UI

User interfaces in Decapod follow the same intent-first methodology as the backend:

```
User Intent → UI State → Component Tree → Render Output
```

The UI is a **projection of state**, not a source of truth. All mutations flow through the control plane.

### 1.2 Core Principles

1. **State at the Center**: UI components render state; they don't own it
2. **Unidirectional Flow**: User actions → Control plane → State update → Re-render
3. **Explicit Over Implicit**: Every interaction has a declared intent
4. **Proof in the UI**: Validation gates surface in the interface

---

## 2. Component Architecture

### 2.1 Component Hierarchy

```
┌─────────────────────────────────────────┐
│  Presentation Layer (Views/Pages)       │
│  - Route-level components               │
│  - Layout containers                    │
└─────────────────────────────────────────┘
                     │
┌─────────────────────────────────────────┐
│  Container Layer (Smart Components)     │
│  - Connect to control plane             │
│  - Manage local UI state                │
│  - Handle user intent                   │
└─────────────────────────────────────────┘
                     │
┌─────────────────────────────────────────┐
│  Component Layer (Dumb Components)      │
│  - Pure render functions                │
│  - Props in, events out                 │
│  - No side effects                      │
└─────────────────────────────────────────┘
                     │
┌─────────────────────────────────────────┐
│  Primitive Layer (Design Tokens)        │
│  - Buttons, inputs, text               │
│  - Theme-aware                          │
│  - Accessibility first                 │
└─────────────────────────────────────────┘
```

### 2.2 Component Registry Schema

```json
{
  "ComponentRegistry": {
    "type": "object",
    "properties": {
      "components": {
        "type": "array",
        "items": {
          "type": "object",
          "required": ["name", "version", "props"],
          "properties": {
            "name": {
              "type": "string",
              "pattern": "^[A-Z][a-zA-Z0-9]+$",
              "description": "PascalCase component name"
            },
            "version": {
              "type": "string",
              "pattern": "^\\d+\\.\\d+\\.\\d+$"
            },
            "category": {
              "type": "string",
              "enum": ["primitive", "composite", "layout", "data-display", "inputs", "feedback"]
            },
            "props": {
              "type": "array",
              "items": {
                "type": "object",
                "required": ["name", "type"],
                "properties": {
                  "name": {"type": "string"},
                  "type": {"type": "string"},
                  "required": {"type": "boolean"},
                  "default": {},
                  "description": {"type": "string"}
                }
              }
            },
            "slots": {
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "name": {"type": "string"},
                  "multiple": {"type": "boolean"},
                  "fallback": {"type": "string"}
                }
              }
            },
            "events": {
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "name": {"type": "string"},
                  "payload": {"type": "string"}
                }
              }
            },
            "accessibility": {
              "type": "object",
              "properties": {
                "role": {"type": "string"},
                "aria": {
                  "type": "object",
                  "additionalProperties": {"type": "string"}
                },
                "keyboard": {
                  "type": "array",
                  "items": {"type": "string"}
                },
                "focusManagement": {
                  "type": "string",
                  "enum": ["focus-trap", "focus-lock", "roving-tabindex", "none"]
                }
              }
            },
            "variants": {
              "type": "array",
              "items": {
                "type": "object",
                "properties": {
                  "name": {"type": "string"},
                  "description": {"type": "string"},
                  "props": {
                    "type": "object",
                    "additionalProperties": {}
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
```

### 2.3 Design Token Schema

```json
{
  "DesignTokens": {
    "colors": {
      "primitive": {
        "white": "#FFFFFF",
        "black": "#000000",
        "blue-50": "#EFF6FF",
        "blue-100": "#DBEAFE",
        "blue-500": "#3B82F6",
        "blue-600": "#2563EB",
        "blue-700": "#1D4ED8",
        "gray-50": "#F9FAFB",
        "gray-100": "#F3F4F6",
        "gray-200": "#E5E7EB",
        "gray-300": "#D1D5DB",
        "gray-400": "#9CA3AF",
        "gray-500": "#6B7280",
        "gray-600": "#4B5563",
        "gray-700": "#374151",
        "gray-800": "#1F2937",
        "gray-900": "#111827"
      },
      "semantic": {
        "background-primary": "{colors.primitive.white}",
        "background-secondary": "{colors.primitive.gray-50}",
        "background-tertiary": "{colors.primitive.gray-100}",
        "text-primary": "{colors.primitive.gray-900}",
        "text-secondary": "{colors.primitive.gray-600}",
        "text-muted": "{colors.primitive.gray-400}",
        "border-default": "{colors.primitive.gray-200}",
        "border-strong": "{colors.primitive.gray-300}",
        "interactive-primary": "{colors.primitive.blue-600}",
        "interactive-hover": "{colors.primitive.blue-700}",
        "interactive-active": "{colors.primitive.blue-700}",
        "success": "#10B981",
        "warning": "#F59E0B",
        "error": "#EF4444",
        "info": "#3B82F6"
      }
    },
    "spacing": {
      "0": "0",
      "1": "0.25rem",
      "2": "0.5rem",
      "3": "0.75rem",
      "4": "1rem",
      "5": "1.25rem",
      "6": "1.5rem",
      "8": "2rem",
      "10": "2.5rem",
      "12": "3rem",
      "16": "4rem",
      "20": "5rem",
      "24": "6rem"
    },
    "typography": {
      "font-family": {
        "sans": "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif",
        "mono": "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace"
      },
      "font-size": {
        "xs": "0.75rem",
        "sm": "0.875rem",
        "base": "1rem",
        "lg": "1.125rem",
        "xl": "1.25rem",
        "2xl": "1.5rem",
        "3xl": "1.875rem",
        "4xl": "2.25rem"
      },
      "font-weight": {
        "normal": "400",
        "medium": "500",
        "semibold": "600",
        "bold": "700"
      },
      "line-height": {
        "tight": "1.25",
        "normal": "1.5",
        "relaxed": "1.625"
      }
    },
    "border-radius": {
      "none": "0",
      "sm": "0.125rem",
      "default": "0.25rem",
      "md": "0.375rem",
      "lg": "0.5rem",
      "xl": "0.75rem",
      "full": "9999px"
    },
    "shadows": {
      "sm": "0 1px 2px 0 rgb(0 0 0 / 0.05)",
      "default": "0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)",
      "md": "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)",
      "lg": "0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)",
      "xl": "0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)"
    }
  }
}
```

---

## 3. Control Plane Integration

### 3.1 Control Plane Adapter Interface

```typescript
// types/control-plane.ts

export interface ControlPlaneAdapter {
  query<T>(command: string, params?: QueryParams): Promise<T>;
  execute(command: string, params?: ExecutionParams): Promise<ExecutionResult>;
  subscribe(event: string, callback: EventHandler): Subscription;
  unsubscribe(subscription: Subscription): void;
}

export interface QueryParams {
  filter?: Record<string, unknown>;
  pagination?: {
    cursor?: string;
    limit?: number;
  };
  sort?: {
    field: string;
    direction: 'asc' | 'desc';
  };
}

export interface ExecutionParams {
  intent: Intent;
  metadata?: Record<string, unknown>;
}

export interface Intent {
  type: string;
  payload: Record<string, unknown>;
  author?: string;
  timestamp?: string;
}

export interface ExecutionResult {
  success: boolean;
  data?: unknown;
  error?: {
    code: string;
    message: string;
    details?: unknown;
  };
  validation?: {
    passed: boolean;
    warnings?: string[];
  };
}

export interface EventHandler {
  (event: ControlPlaneEvent): void | Promise<void>;
}

export interface ControlPlaneEvent {
  type: string;
  timestamp: string;
  data: unknown;
  correlationId?: string;
}

export interface Subscription {
  id: string;
  event: string;
  handler: EventHandler;
}
```

### 3.2 React Integration Hook

```typescript
// hooks/useControlPlane.ts
import { useCallback, useEffect, useState } from 'react';
import type { 
  ControlPlaneAdapter, 
  QueryParams, 
  ExecutionParams,
  ExecutionResult 
} from '@/types/control-plane';

interface UseControlPlaneOptions {
  adapter: ControlPlaneAdapter;
  onError?: (error: Error) => void;
  onEvent?: (event: ControlPlaneEvent) => void;
}

export function useControlPlane(options: UseControlPlaneOptions) {
  const { adapter, onError, onEvent } = options;
  const [isLoading, setIsLoading] = useState(false);

  const query = useCallback(async <T>(
    command: string, 
    params?: QueryParams
  ): Promise<T> => {
    setIsLoading(true);
    try {
      const result = await adapter.query<T>(command, params);
      return result;
    } catch (error) {
      onError?.(error as Error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [adapter, onError]);

  const execute = useCallback(async (
    command: string,
    params?: ExecutionParams
  ): Promise<ExecutionResult> => {
    setIsLoading(true);
    try {
      const result = await adapter.execute(command, params);
      return result;
    } catch (error) {
      onError?.(error as Error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, [adapter, onError]);

  return {
    query,
    execute,
    isLoading,
  };
}

// Usage in a component
function TodoList({ adapter }: { adapter: ControlPlaneAdapter }) {
  const { query, execute, isLoading } = useControlPlane({
    adapter,
    onError: (error) => console.error('Control plane error:', error),
  });

  const [todos, setTodos] = useState<Todo[]>([]);

  useEffect(() => {
    query<Todo[]>('todo.list').then(setTodos);
  }, [query]);

  const handleComplete = async (todoId: string) => {
    await execute('todo.done', {
      intent: {
        type: 'TODO_COMPLETE',
        payload: { todoId },
      },
    });
  };

  if (isLoading && todos.length === 0) {
    return <Skeleton />;
  }

  return (
    <ul>
      {todos.map((todo) => (
        <TodoItem
          key={todo.id}
          todo={todo}
          onComplete={() => handleComplete(todo.id)}
        />
      ))}
    </ul>
  );
}
```

---

## 4. State Management Patterns

### 4.1 State Classification Matrix

| Type | Location | Examples | Persistence | Synchronization |
|------|----------|-----------|-------------|-----------------|
| Domain State | Control plane | TODOs, claims, tasks | Required | Real-time sync |
| UI State | Component local | Modal open, tab active | None | None |
| URL State | Browser URL | Filters, pagination | Bookmarkable | Browser back/forward |
| Session State | Server + Client | Auth token, preferences | Required | On refresh |
| Form State | Component local | Input values, validation | Partial | Depends on requirements |

### 4.2 URL State Management

```typescript
// hooks/useURLState.ts
import { useSearchParams, useRouter, usePathname } from 'next/navigation';
import { useCallback, useMemo } from 'react';

interface URLStateOptions<T> {
  defaultValue: T;
  serialize: (value: T) => string;
  deserialize: (value: string) => T;
}

export function useURLState<T>(
  key: string,
  options: URLStateOptions<T>
): [T, (value: T) => void, () => void] {
  const { defaultValue, serialize, deserialize } = options;
  const searchParams = useSearchParams();
  const router = useRouter();
  const pathname = usePathname();

  const value = useMemo(() => {
    const param = searchParams.get(key);
    if (param === null) return defaultValue;
    try {
      return deserialize(param);
    } catch {
      return defaultValue;
    }
  }, [searchParams, key, defaultValue, deserialize]);

  const setValue = useCallback((newValue: T) => {
    const newParams = new URLSearchParams(searchParams.toString());
    newParams.set(key, serialize(newValue));
    router.push(`${pathname}?${newParams.toString()}`, { scroll: false });
  }, [key, serialize, router, pathname, searchParams]);

  const clearValue = useCallback(() => {
    const newParams = new URLSearchParams(searchParams.toString());
    newParams.delete(key);
    router.push(`${pathname}?${newParams.toString()}`, { scroll: false });
  }, [key, router, pathname, searchParams]);

  return [value, setValue, clearValue];
}

// Usage
function Filters() {
  const [filters, setFilters] = useURLState('filters', {
    defaultValue: { status: 'all', category: 'all' },
    serialize: (value) => btoa(JSON.stringify(value)),
    deserialize: (value) => JSON.parse(atob(value)),
  });

  // ...
}
```

---

## 5. Validation & Proof Visualization

### 5.1 Validation Status Component

```typescript
// components/ValidationStatus.tsx
import { CheckCircle, XCircle, AlertTriangle, Info } from 'lucide-react';

interface ValidationStatusProps {
  status: 'pass' | 'fail' | 'warning' | 'info';
  count?: {
    passed?: number;
    failed?: number;
    warnings?: number;
  };
  onClick?: () => void;
}

export function ValidationStatus({ status, count, onClick }: ValidationStatusProps) {
  const config = {
    pass: {
      icon: CheckCircle,
      color: 'text-green-600',
      bgColor: 'bg-green-50',
      label: 'Pass',
    },
    fail: {
      icon: XCircle,
      color: 'text-red-600',
      bgColor: 'bg-red-50',
      label: 'Fail',
    },
    warning: {
      icon: AlertTriangle,
      color: 'text-yellow-600',
      bgColor: 'bg-yellow-50',
      label: 'Warning',
    },
    info: {
      icon: Info,
      color: 'text-blue-600',
      bgColor: 'bg-blue-50',
      label: 'Info',
    },
  };

  const { icon: Icon, color, bgColor, label } = config[status];

  return (
    <button
      type="button"
      onClick={onClick}
      className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-full ${bgColor} ${color}`}
    >
      <Icon size={16} />
      <span className="font-medium">{label}</span>
      {count && (
        <span className="text-sm opacity-75">
          {count.failed !== undefined && `${count.failed} failed`}
          {count.warnings !== undefined && `${count.warnings} warnings`}
          {count.passed !== undefined && `${count.passed} passed`}
        </span>
      )}
    </button>
  );
}
```

### 5.2 Validation Gate Display

```typescript
// components/ValidationGate.tsx
interface ValidationGateProps {
  gate: {
    name: string;
    status: 'pass' | 'fail' | 'warning' | 'info';
    message?: string;
    details?: unknown;
  };
  expanded?: boolean;
}

export function ValidationGate({ gate, expanded = false }: ValidationGateProps) {
  return (
    <div className="border rounded-lg overflow-hidden">
      <div className="flex items-center gap-3 p-3 bg-gray-50">
        <ValidationStatus status={gate.status} />
        <span className="font-medium">{gate.name}</span>
        {gate.message && (
          <span className="text-gray-600 text-sm">{gate.message}</span>
        )}
      </div>
      
      {expanded && gate.details && (
        <div className="p-3 border-t bg-white">
          <pre className="text-xs overflow-auto">
            {JSON.stringify(gate.details, null, 2)}
          </pre>
        </div>
      )}
    </div>
  );
}
```

---

## 6. Accessibility Implementation

### 6.1 Focus Management

```typescript
// hooks/useFocusManagement.ts
import { useRef, useEffect, useCallback } from 'react';

export function useFocusTrap(isActive: boolean) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!isActive || !containerRef.current) return;

    const container = containerRef.current;
    const focusableElements = container.querySelectorAll<HTMLElement>(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );

    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return;

      if (e.shiftKey && document.activeElement === firstElement) {
        e.preventDefault();
        lastElement?.focus();
      } else if (!e.shiftKey && document.activeElement === lastElement) {
        e.preventDefault();
        firstElement?.focus();
      }
    };

    container.addEventListener('keydown', handleKeyDown);
    firstElement?.focus();

    return () => container.removeEventListener('keydown', handleKeyDown);
  }, [isActive]);

  return containerRef;
}

export function useRovingTabIndex<T extends HTMLElement>(
  items: unknown[],
  options?: {
    orientation?: 'horizontal' | 'vertical' | 'both';
  }
) {
  const containerRef = useRef<T>(null);
  const currentIndex = useRef(0);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const items = container.querySelectorAll<HTMLElement>(
      '[data-roving-index]:not([disabled])'
    );

    items.forEach((item, index) => {
      item.setAttribute('tabindex', index === currentIndex.current ? '0' : '-1');
    });

    const handleKeyDown = (e: KeyboardEvent) => {
      const { orientation = 'vertical' } = options || {};

      let nextIndex = currentIndex.current;
      const maxIndex = items.length - 1;

      switch (e.key) {
        case 'ArrowDown':
          if (orientation === 'horizontal') return;
          e.preventDefault();
          nextIndex = Math.min(nextIndex + 1, maxIndex);
          break;
        case 'ArrowUp':
          if (orientation === 'horizontal') return;
          e.preventDefault();
          nextIndex = Math.max(nextIndex - 1, 0);
          break;
        case 'ArrowRight':
          if (orientation === 'vertical') return;
          e.preventDefault();
          nextIndex = Math.min(nextIndex + 1, maxIndex);
          break;
        case 'ArrowLeft':
          if (orientation === 'vertical') return;
          e.preventDefault();
          nextIndex = Math.max(nextIndex - 1, 0);
          break;
        case 'Home':
          e.preventDefault();
          nextIndex = 0;
          break;
        case 'End':
          e.preventDefault();
          nextIndex = maxIndex;
          break;
        default:
          return;
      }

      if (nextIndex !== currentIndex.current) {
        items[currentIndex.current]?.setAttribute('tabindex', '-1');
        items[nextIndex]?.setAttribute('tabindex', '0');
        items[nextIndex]?.focus();
        currentIndex.current = nextIndex;
      }
    };

    container.addEventListener('keydown', handleKeyDown);
    return () => container.removeEventListener('keydown', handleKeyDown);
  }, [items, options]);

  return containerRef;
}
```

### 6.2 Live Region Announcements

```typescript
// components/LiveRegion.tsx
import { useEffect, useState, createContext, useContext } from 'react';

interface LiveRegionContextValue {
  announce: (message: string, politeness?: 'polite' | 'assertive') => void;
}

const LiveRegionContext = createContext<LiveRegionContextValue | null>(null);

export function LiveRegionProvider({ children }: { children: React.ReactNode }) {
  const [politeMessage, setPoliteMessage] = useState('');
  const [assertiveMessage, setAssertiveMessage] = useState('');

  const announce = (message: string, politeness: 'polite' | 'assertive' = 'polite') => {
    if (politeness === 'assertive') {
      setAssertiveMessage(message);
      setTimeout(() => setAssertiveMessage(''), 1000);
    } else {
      setPoliteMessage(message);
      setTimeout(() => setPoliteMessage(''), 1000);
    }
  };

  return (
    <LiveRegionContext.Provider value={{ announce }}>
      {children}
      <div
        role="status"
        aria-live="polite"
        aria-atomic="true"
        className="sr-only"
      >
        {politeMessage}
      </div>
      <div
        role="alert"
        aria-live="assertive"
        aria-atomic="true"
        className="sr-only"
      >
        {assertiveMessage}
      </div>
    </LiveRegionContext.Provider>
  );
}

export function useLiveRegion() {
  const context = useContext(LiveRegionContext);
  if (!context) {
    throw new Error('useLiveRegion must be used within LiveRegionProvider');
  }
  return context;
}
```

---

## 7. Error Handling

### 7.1 Error Boundary

```typescript
// components/ErrorBoundary.tsx
import { Component, type ReactNode, type ErrorInfo } from 'react';

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('ErrorBoundary caught:', error, errorInfo);
    this.props.onError?.(error, errorInfo);
    
    // Report to error tracking service
    reportError(error, { componentStack: errorInfo.componentStack });
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }
      
      return (
        <div className="p-6 max-w-md mx-auto">
          <h1 className="text-xl font-bold text-red-600">Something went wrong</h1>
          <p className="mt-2 text-gray-600">
            We're sorry for the inconvenience. Please try refreshing the page.
          </p>
          <pre className="mt-4 p-4 bg-gray-100 rounded text-xs overflow-auto">
            {this.state.error?.message}
          </pre>
        </div>
      );
    }

    return this.props.children;
  }
}
```

### 7.2 Error State Patterns

```typescript
// types/error-state.ts
export interface UIError {
  id: string;
  type: 'validation' | 'network' | 'control_plane' | 'unknown';
  code: string;
  message: string;
  recoverable: boolean;
  retryable?: boolean;
  action?: {
    label: string;
    onClick: () => void;
  };
  timestamp: Date;
  context?: Record<string, unknown>;
}

export function createErrorState(
  error: unknown,
  fallbackMessage = 'An unexpected error occurred'
): UIError {
  if (error instanceof ControlPlaneError) {
    return {
      id: crypto.randomUUID(),
      type: 'control_plane',
      code: error.code,
      message: error.message,
      recoverable: error.recoverable,
      retryable: error.retryable,
      timestamp: new Date(),
      context: error.context,
    };
  }
  
  if (error instanceof NetworkError) {
    return {
      id: crypto.randomUUID(),
      type: 'network',
      code: error.code || 'NETWORK_ERROR',
      message: 'Network connection failed. Please check your internet connection.',
      recoverable: true,
      retryable: true,
      timestamp: new Date(),
    };
  }
  
  return {
    id: crypto.randomUUID(),
    type: 'unknown',
    code: 'UNKNOWN_ERROR',
    message: fallbackMessage,
    recoverable: false,
    timestamp: new Date(),
  };
}
```

---

## 8. Performance Patterns

### 8.1 Virtualization

```typescript
// hooks/useVirtualList.ts
import { useState, useEffect, useRef, useCallback } from 'react';

interface VirtualListOptions<T> {
  items: T[];
  itemHeight: number | ((index: number) => number);
  containerHeight: number;
  overscan?: number;
}

export function useVirtualList<T>(options: VirtualListOptions<T>) {
  const { items, itemHeight, containerHeight, overscan = 3 } = options;
  
  const [scrollTop, setScrollTop] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);

  const getItemHeight = useCallback(
    (index: number) => {
      return typeof itemHeight === 'function' ? itemHeight(index) : itemHeight;
    },
    [itemHeight]
  );

  const totalHeight = items.reduce(
    (sum, _, index) => sum + getItemHeight(index),
    0
  );

  const visibleRange = useCallback(() => {
    let startIndex = 0;
    let accumulated = 0;

    for (let i = 0; i < items.length; i++) {
      const height = getItemHeight(i);
      if (accumulated + height >= scrollTop) {
        startIndex = Math.max(0, i - overscan);
        break;
      }
      accumulated += height;
    }

    let endIndex = startIndex;
    accumulated = items
      .slice(0, startIndex)
      .reduce((sum, _, i) => sum + getItemHeight(i), 0);

    while (accumulated < scrollTop + containerHeight && endIndex < items.length) {
      accumulated += getItemHeight(endIndex);
      endIndex++;
    }

    endIndex = Math.min(endIndex + overscan, items.length - 1);

    return { startIndex, endIndex };
  }, [items, scrollTop, containerHeight, getItemHeight, overscan]);

  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  }, []);

  const virtualItems = useCallback(() => {
    const { startIndex, endIndex } = visibleRange();
    
    let offset = items
      .slice(0, startIndex)
      .reduce((sum, _, i) => sum + getItemHeight(i), 0);

    const result = [];
    for (let i = startIndex; i <= endIndex; i++) {
      result.push({
        index: i,
        item: items[i],
        offset,
        height: getItemHeight(i),
      });
      offset += getItemHeight(i);
    }

    return result;
  }, [visibleRange, items, getItemHeight]);

  return {
    containerRef,
    totalHeight,
    virtualItems: virtualItems(),
    handleScroll,
  };
}
```

### 8.2 Debouncing

```typescript
// hooks/useDebounce.ts
import { useState, useEffect, useRef } from 'react';

export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => clearTimeout(timer);
  }, [value, delay]);

  return debouncedValue;
}

export function useDebouncedCallback<T extends (...args: unknown[]) => unknown>(
  callback: T,
  delay: number
): [(...args: Parameters<T>) => void, boolean] {
  const timeoutRef = useRef<NodeJS.Timeout>();
  const callbackRef = useRef(callback);
  
  // Update callback ref when callback changes
  useEffect(() => {
    callbackRef.current = callback;
  }, [callback]);

  const debouncedCallback = (...args: Parameters<T>) => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    
    timeoutRef.current = setTimeout(() => {
      callbackRef.current(...args);
    }, delay);
  };

  const cancel = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
  }, []);

  return [debouncedCallback, cancel];
}
```

---

## 9. Testing Strategy

### 9.1 Component Testing

```typescript
// components/Button.test.tsx
import { render, screen, userEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { Button } from './Button';

describe('Button', () => {
  it('renders with correct label', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByRole('button', { name: 'Click me' })).toBeInTheDocument();
  });

  it('handles click events', async () => {
    const onClick = vi.fn();
    const user = userEvent.setup();
    
    render(<Button onClick={onClick}>Click me</Button>);
    await user.click(screen.getByRole('button'));
    
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('is disabled when disabled prop is true', () => {
    render(<Button disabled>Disabled</Button>);
    expect(screen.getByRole('button')).toBeDisabled();
  });

  it('shows loading state', () => {
    render(<Button isLoading loadingText="Saving...">Save</Button>);
    expect(screen.getByText('Saving...')).toBeInTheDocument();
  });
});

describe('ValidationGate', () => {
  it('renders pass status correctly', () => {
    render(<ValidationGate gate={{ name: 'Test Gate', status: 'pass' }} />);
    expect(screen.getByText('Pass')).toBeInTheDocument();
  });

  it('shows expanded details when expanded', () => {
    const gate = {
      name: 'Test Gate',
      status: 'fail' as const,
      message: 'Test failed',
      details: { reason: 'validation_error' },
    };
    
    render(<ValidationGate gate={gate} expanded />);
    expect(screen.getByText(/"reason": "validation_error"/)).toBeInTheDocument();
  });
});
```

---

## 10. Anti-Patterns

| Anti-Pattern | Specific Failure Mode | Prevention |
|--------------|----------------------|------------|
| **State duplication** | UI state drifts from control plane | Single source of truth |
| **Prop drilling** | Complex maintenance | Context, composition |
| **No error boundaries** | White screen of death | ErrorBoundary component |
| **Inline styles** | Inconsistency, maintenance issues | Design tokens |
| **Magic numbers** | Inconsistent spacing/sizing | Design tokens |
| **No focus management** | Keyboard navigation broken | useFocusTrap |
| **Missing ARIA labels** | Screen reader unusable | Accessibility checks |
| **Synchronous state updates** | Stale UI | Async updates with loading states |
| **Large component trees** | Slow renders | Code splitting, lazy loading |
| **No skeleton loading** | Layout shift, blank screens | Skeleton components |

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**

### Architecture (This Section)
- `architecture/FRONTEND.md` - Frontend architecture patterns
- `architecture/WEB.md` - Web architecture patterns
- `architecture/SECURITY.md` - Security architecture

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition
- `specs/SECURITY.md` - Security contract

### Interface Contracts
- `interfaces/CONTROL_PLANE.md` - Control plane adapter
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/STORE_MODEL.md` - Store semantics

### Methodology
- `methodology/ARCHITECTURE.md` - Architecture methodology
- `methodology/UI_COMPONENTS.md` - Component design