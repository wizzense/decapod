# FRONTEND.md - Frontend Architecture (DENSE)

**Authority:** guidance (frontend patterns, performance, and user experience)
**Layer:** Guides
**Binding:** No
**Scope:** frontend architecture, performance optimization, and UX patterns
**Non-goals:** specific framework tutorials, visual design guidelines

---

## 1. Frontend Architecture Principles

### 1.1 Core Web Vitals Requirements

| Metric | Good | Needs Improvement | Poor |
|--------|------|-------------------|------|
| LCP (Largest Contentful Paint) | < 2.5s | 2.5s - 4s | > 4s |
| FID (First Input Delay) | < 100ms | 100ms - 300ms | > 300ms |
| CLS (Cumulative Layout Shift) | < 0.1 | 0.1 - 0.25 | > 0.25 |
| INP (Interaction to Next Paint) | < 200ms | 200ms - 500ms | > 500ms |
| TTFB (Time to First Byte) | < 800ms | 800ms - 1800ms | > 1800ms |

### 1.2 Performance Budget

```json
{
  "PerformanceBudget": {
    "total_bundle_size": {
      "initial": "250KB",
      "critical_path": "50KB",
      "gzip": true
    },
    "largest_contentful_paint": {
      "target_ms": 2500,
      "budget_allocation": {
        "server_response": 600,
        "document_transfer": 300,
        "parse_compile": 500,
        "render_blocking": 200,
        "long_tasks": 500,
        "user_interaction": 400
      }
    },
    "interaction_to_next_paint": {
      "target_ms": 200,
      "components": {
        "click_response": 100,
        "input_response": 50,
        "animation_frame": 50
      }
    }
  }
}
```

### 1.3 Production Mindset
The frontend is not a layer — it is the product. Every decision that degrades the user experience degrades the product itself:

- **Time-to-interactive is a revenue metric:** A bloated JavaScript bundle has a direct, measurable impact on conversion and retention. Every new dependency must justify its payload weight. If a library costs 200KB to format a date, replace it with 5 lines.
- **Framework stability over novelty:** Rewriting the frontend every time a new framework trends is a net loss. Choose a mature, well-supported ecosystem and hold it. Innovation belongs in the user experience and product capability, not the build toolchain.
- **Accessibility is a correctness requirement, not a backlog item:** If a core flow cannot be completed with a keyboard and screen reader, the feature is defective. This is both an ethical and legal obligation, and it must be verified before any flow is marked complete.
- **Standardized components over bespoke CSS:** A consistent, accessible component library is a force multiplier. Custom widget implementations for standard patterns (buttons, modals, selects) accumulate accessibility debt and design drift. Use and maintain a shared system.
- **State locality reduces complexity:** The largest source of frontend complexity is state that lives farther from its use site than necessary. Reach for global state only when multiple disconnected components strictly require synchronization. Local and URL state should be the defaults.
- **Choose the rendering model for the use case:** SSR and SSG are the correct defaults for content-heavy pages and SEO-critical surfaces. Pay the cost of a full SPA only when the interface genuinely requires app-level interactivity that cannot be achieved otherwise.
- **Server-state libraries are the standard:** Manual `useEffect` for data fetching is error-prone and widely superseded. Libraries like React Query and SWR handle caching, deduplication, background refresh, and error states correctly. Use them.
- **Monitor bundle size as a first-class metric:** Tree-shaking must be verified, not assumed. Bundle analysis should run in CI. Size regressions are caught at PR review, not discovered when performance degrades in production.

---

## 2. Rendering Strategies

### 2.1 Rendering Strategy Selection Matrix

| Strategy | Initial Load | Time to Interactive | SEO | Dynamic Content | Complex Interactivity |
|----------|-------------|-------------------|-----|----------------|----------------------|
| SSG | Fastest | Fast | Excellent | Poor | Limited |
| SSR | Fast | Fast | Excellent | Good | Limited |
| ISR | Fast | Fast | Excellent | Good | Limited |
| SPA | Slow | Slow | Poor | Excellent | Excellent |
| Islands | Fast | Fast | Good | Good | Selective |

### 2.2 Next.js SSR Configuration

```typescript
// next.config.ts - Production Next.js Configuration
import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  // Core settings
  reactStrictMode: true,
  swcMinify: true,
  
  // Output configuration
  output: 'standalone',
  
  // Image optimization
  images: {
    formats: ['image/avif', 'image/webp'],
    minimumCacheTTL: 60 * 60 * 24 * 365, // 1 year
    remotePatterns: [
      {
        protocol: 'https',
        hostname: '**.example.com',
      },
    ],
    deviceSizes: {
      sm: 640,
      md: 768,
      lg: 1024,
      xl: 1280,
      xxl: 1920,
    },
  },
  
  // Headers for caching and security
  async headers() {
    return [
      {
        source: '/:path*',
        headers: [
          { key: 'X-DNS-Prefetch-Control', value: 'on' },
          { key: 'Strict-Transport-Security', value: 'max-age=63072000; includeSubDomains; preload' },
          { key: 'X-Content-Type-Options', value: 'nosniff' },
          { key: 'X-Frame-Options', value: 'SAMEORIGIN' },
          { key: 'X-XSS-Protection', value: '1; mode=block' },
        ],
      },
      {
        source: '/static/:path*',
        headers: [
          { key: 'Cache-Control', value: 'public, max-age=31536000, immutable' },
        ],
      },
      {
        source: '/api/:path*',
        headers: [
          { key: 'Cache-Control', value: 'private, no-cache, no-store, must-revalidate' },
        ],
      },
    ];
  },
  
  // Redirects
  async redirects() {
    return [
      {
        source: '/old-path/:slug*',
        destination: '/new-path/:slug*',
        permanent: true,
      },
    ];
  },
  
  // Rewrites for API proxy
  async rewrites() {
    return [
      {
        source: '/api/proxy/:path*',
        destination: 'https://external-api.example.com/:path*',
      },
    ];
  },
  
  // Bundle analyzer
  bundleAnalysis: {
    enabled: process.env.ANALYZE === 'true',
  },
  
  // Compiler options
  compiler: {
    removeConsole: process.env.NODE_ENV === 'production',
    optimizePackageImports: ['lucide-react', '@radix-ui/react-icons'],
  },
  
  // Experimental features
  experimental: {
    optimizeCss: true,
    scrollRestoration: true,
  },
};
```

### 2.3 Astro Islands Configuration

```javascript
// astro.config.mjs
import { defineConfig } from 'astro/config';
import react from '@astrojs/react';
import tailwind from '@astrojs/tailwind';
import vercel from '@astrojs/vercel/serverless';

export default defineConfig({
  output: 'hybrid',
  adapter: vercel({
    edgeMiddleware: true,
  }),
  integrations: [
    react(),
    tailwind(),
  ],
  
  // Vite optimization
  vite: {
    build: {
      cssCodeSplit: true,
      rollupOptions: {
        output: {
          manualChunks: {
            vendor: ['react', 'react-dom'],
            icons: ['lucide-react'],
          },
        },
      },
    },
    optimizeDeps: {
      include: ['react', 'react-dom'],
    },
  },
  
  // Hydration strategy
  hydration: {
    prerenderInjectsGoTo: false,
    clientDirective: 'client:visible',
  },
});
```

---

## 3. State Management

### 3.1 State Management Decision Tree

```
Need to manage state?
|
+-- Is it server data (API responses)?
|   +-- Use React Query / SWR / TanStack Query
|
+-- Is it URL state (filters, pagination)?
|   +-- Use URL state (nuqs, history API)
|
+-- Is it form state?
|   +-- Use React Hook Form + Zod validation
|
+-- Is it UI state (modals, tabs)?
|   +-- Use useState in nearest container
|
+-- Is it cross-component state?
|   +-- Use Context (simple) or Zustand (complex)
|
+-- Is it global (auth, theme)?
|   +-- Use Zustand with persistence
```

### 3.2 React Query Configuration

```typescript
// lib/api.ts
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { httpBatchLink } from '@trpc/client';
import createTRPCProxyClient from '@trpc/react-query';

// React Query client configuration
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Stale time: how long data is considered fresh
      staleTime: 5 * 60 * 1000, // 5 minutes
      
      // Cache time: how long unused data stays in cache
      gcTime: 10 * 60 * 1000, // 10 minutes
      
      // Retry configuration
      retry: (failureCount, error) => {
        // Don't retry on 4xx errors
        if (error.status >= 400 && error.status < 500) {
          return false;
        }
        // Retry up to 3 times with exponential backoff
        return failureCount < 3;
      },
      
      // Background refetch on window focus
      refetchOnWindowFocus: 'smart',
      
      // Refetch on reconnect
      refetchOnReconnect: 'always',
    },
    
    mutations: {
      // Retry failed mutations once
      retry: 1,
    },
  },
});

// TRPC client with React Query
export const trpc = createTRPCProxyClient({
  links: [
    httpBatchLink({
      url: '/api/trpc',
      headers: () => {
        const token = getAuthToken();
        return token ? { Authorization: `Bearer ${token}` } : {};
      },
    }),
  ],
});

// Hook for data fetching
export function useUsers(filter?: UserFilter) {
  return trpc.users.list.useQuery(
    { filter },
    {
      // Keep previous data while fetching new
      placeholderData: (previousData) => previousData,
      
      // Transform data
      select: (data) => ({
        ...data,
        users: data.users.filter(u => u.status === 'active'),
      }),
    }
  );
}

// Mutation hook
export function useCreateUser() {
  return trpc.users.create.useMutation({
    onSuccess: (data) => {
      // Invalidate and refetch users list
      queryClient.invalidateQueries({ queryKey: ['users', 'list'] });
      
      // Navigate to new user
      router.push(`/users/${data.id}`);
    },
    onError: (error) => {
      toast.error(error.message);
    },
  });
}
```

### 3.3 Zustand Store Configuration

```typescript
// stores/auth.store.ts
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';

interface User {
  id: string;
  email: string;
  name: string;
  roles: string[];
}

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  
  // Actions
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  refreshToken: () => Promise<void>;
  updateProfile: (updates: Partial<User>) => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      token: null,
      isAuthenticated: false,
      
      login: async (email, password) => {
        const response = await api.auth.login({ email, password });
        set({
          user: response.user,
          token: response.token,
          isAuthenticated: true,
        });
      },
      
      logout: () => {
        set({
          user: null,
          token: null,
          isAuthenticated: false,
        });
      },
      
      refreshToken: async () => {
        const { token } = get();
        if (!token) return;
        
        const response = await api.auth.refresh({ token });
        set({ token: response.token });
      },
      
      updateProfile: (updates) => {
        const { user } = get();
        if (!user) return;
        
        set({
          user: { ...user, ...updates },
        });
      },
    }),
    {
      name: 'auth-storage',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        token: state.token,
        user: state.user,
        isAuthenticated: state.isAuthenticated,
      }),
    }
  )
);

// Theme store with localStorage persistence
interface ThemeState {
  mode: 'light' | 'dark' | 'system';
  resolvedMode: 'light' | 'dark';
  setMode: (mode: ThemeState['mode']) => void;
}

export const useThemeStore = create<ThemeState>()((set, get) => ({
  mode: 'system',
  resolvedMode: 'light',
  
  setMode: (mode) => {
    const resolved = mode === 'system'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
      : mode;
    
    set({ mode, resolvedMode: resolved });
    document.documentElement.setAttribute('data-theme', resolved);
  },
}));
```

### 3.4 React Hook Form + Zod

```typescript
// lib/validation.ts
import { z } from 'zod';

export const createUserSchema = z.object({
  email: z
    .string()
    .email('Invalid email address')
    .min(1, 'Email is required')
    .max(254, 'Email is too long'),
  
  password: z
    .string()
    .min(12, 'Password must be at least 12 characters')
    .regex(/[A-Z]/, 'Password must contain at least one uppercase letter')
    .regex(/[a-z]/, 'Password must contain at least one lowercase letter')
    .regex(/[0-9]/, 'Password must contain at least one number')
    .regex(/[^A-Za-z0-9]/, 'Password must contain at least one special character'),
  
  confirmPassword: z.string(),
  displayName: z
    .string()
    .min(1, 'Display name is required')
    .max(100, 'Display name is too long')
    .regex(/^[a-zA-Z0-9\s\-']+$/, 'Display name contains invalid characters'),
  
  roles: z.array(z.enum(['admin', 'editor', 'viewer'])),
  acceptTerms: z.literal(true, {
    errorMap: () => ({ message: 'You must accept the terms' }),
  }),
}).refine((data) => data.password === data.confirmPassword, {
  message: 'Passwords do not match',
  path: ['confirmPassword'],
});

export type CreateUserForm = z.infer<typeof createUserSchema>;

// Component usage
export function CreateUserForm() {
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
    watch,
  } = useForm<CreateUserForm>({
    resolver: zodResolver(createUserSchema),
    defaultValues: {
      roles: ['viewer'],
    },
  });
  
  const password = watch('password');
  
  const onSubmit = async (data: CreateUserForm) => {
    await createUser(data);
  };
  
  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <input {...register('email')} type="email" />
      {errors.email && <span>{errors.email.message}</span>}
      
      <input {...register('password')} type="password" />
      {errors.password && <span>{errors.password.message}</span>}
      
      <input 
        {...register('confirmPassword')} 
        type="password"
        aria-describedby="confirmPassword-error"
      />
      {errors.confirmPassword && (
        <span id="confirmPassword-error">{errors.confirmPassword.message}</span>
      )}
      
      <input {...register('displayName')} />
      {errors.displayName && <span>{errors.displayName.message}</span>}
      
      <select {...register('roles')}>
        <option value="viewer">Viewer</option>
        <option value="editor">Editor</option>
        <option value="admin">Admin</option>
      </select>
      
      <input type="checkbox" {...register('acceptTerms')} />
      {errors.acceptTerms && <span>{errors.acceptTerms.message}</span>}
      
      <button type="submit" disabled={isSubmitting}>
        {isSubmitting ? 'Creating...' : 'Create User'}
      </button>
    </form>
  );
}
```

---

## 4. Performance Optimization

### 4.1 Code Splitting Configuration

```typescript
// next.config.ts - Route-based code splitting
const nextConfig = {
  // Automatic page-based splitting (default)
  
  // Manual dynamic imports for components
  webpack: (config, { isServer }) => {
    // Split large dependencies
    config.optimization.splitChunks = {
      chunks: 'all',
      maxInitialRequests: 25,
      minSize: 20000,
      cacheGroups: {
        // React ecosystem
        react: {
          test: /[\\/]node_modules[\\/](react|react-dom)[\\/]/,
          name: 'react',
          chunks: 'all',
          priority: 40,
        },
        
        // UI libraries
        ui: {
          test: /[\\/]node_modules[\\/](@radix-ui|lucide)[\\/]/,
          name: 'ui',
          chunks: 'all',
          priority: 30,
        },
        
        // Date handling
        dateFns: {
          test: /[\\/]node_modules[\\/](date-fns|moment|luxon)[\\/]/,
          name: 'date-fns',
          chunks: 'all',
          priority: 20,
        },
        
        // Vendor chunk for remaining node_modules
        vendors: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          chunks: 'all',
          priority: 10,
        },
      },
    };
    
    return config;
  },
};

// Component-level code splitting
const HeavyChart = dynamic(() => import('./components/HeavyChart'), {
  loading: () => <Skeleton />,
  ssr: false,
  suspense: true,
});

const Modal = dynamic(() => import('./components/Modal'), {
  loading: () => null, // Don't show anything while loading
});
```

### 4.2 Bundle Analyzer Setup

```typescript
// scripts/analyze-bundle.ts
import { bundleAnalysis } from './next.config';

export async function analyzeBundle() {
  const { default: stats } = await import('./.next/static/chunks/stats.json');
  
  // Group chunks by size
  const chunks = stats.modules
    .filter(m => m.chunks && m.chunks.length > 0)
    .map(m => ({
      name: m.name,
      size: m.size,
      chunks: m.chunks,
    }))
    .sort((a, b) => b.size - a.size);
  
  // Find oversized chunks
  const oversizeThreshold = 100 * 1024; // 100KB
  const oversized = chunks.filter(c => c.size > oversizeThreshold);
  
  if (oversized.length > 0) {
    console.warn('Oversized chunks detected:');
    oversized.forEach(c => {
      console.warn(`  ${c.name}: ${(c.size / 1024).toFixed(1)}KB`);
    });
  }
  
  return { chunks, oversized };
}

// CI integration
if (process.env.CI) {
  const { chunks } = await analyzeBundle();
  const totalSize = chunks.reduce((sum, c) => sum + c.size, 0);
  
  // Fail CI if total exceeds budget
  const budget = 250 * 1024; // 250KB
  if (totalSize > budget) {
    throw new Error(`Bundle size ${(totalSize / 1024).toFixed(1)}KB exceeds budget ${(budget / 1024).toFixed(1)}KB`);
  }
}
```

### 4.3 Image Optimization

```tsx
// components/OptimizedImage.tsx
import Image from 'next/image';

interface ProductImageProps {
  src: string;
  alt: string;
  width: number;
  height: number;
  priority?: boolean;
}

export function ProductImage({ src, alt, width, height, priority }: ProductImageProps) {
  return (
    <Image
      src={src}
      alt={alt}
      width={width}
      height={height}
      priority={priority}
      
      // Responsive sizes
      sizes="(max-width: 640px) 100vw,
             (max-width: 1024px) 50vw,
             33vw"
      
      // Placeholder while loading
      placeholder="blur"
      blurDataURL={generateBlurDataURL(src)}
      
      // Quality settings
      quality={80}
      
      // Fill for aspect ratio containers
      fill={false}
    />
  );
}

// Lazy loaded background image
export function LazyBackgroundImage({ src, children }: { src: string; children: React.ReactNode }) {
  const [loaded, setLoaded] = useState(false);
  
  return (
    <div
      style={{
        backgroundImage: loaded ? `url(${src})` : 'none',
        transition: 'opacity 0.3s ease',
      }}
    >
      <Image
        src={src}
        alt=""
        fill
        style={{ objectFit: 'cover', opacity: 0 }}
        onLoad={() => setLoaded(true)}
        loading="lazy"
      />
      {children}
    </div>
  );
}
```

---

## 5. Component Patterns

### 5.1 Component Architecture

```typescript
// components/ui/Button.tsx
import { forwardRef } from 'react';
import { cn } from '@/lib/utils';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'destructive';
  size?: 'sm' | 'md' | 'lg' | 'icon';
  isLoading?: boolean;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      className,
      variant = 'primary',
      size = 'md',
      isLoading = false,
      leftIcon,
      rightIcon,
      disabled,
      children,
      ...props
    },
    ref
  ) => {
    return (
      <button
        ref={ref}
        disabled={disabled || isLoading}
        className={cn(
          // Base styles
          'inline-flex items-center justify-center gap-2 font-medium rounded-lg',
          'transition-colors duration-200',
          'focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2',
          'disabled:opacity-50 disabled:cursor-not-allowed',
          
          // Variants
          {
            'bg-blue-600 text-white hover:bg-blue-700 focus-visible:ring-blue-500':
              variant === 'primary',
            'bg-gray-100 text-gray-900 hover:bg-gray-200 focus-visible:ring-gray-500':
              variant === 'secondary',
            'border border-gray-300 bg-white hover:bg-gray-50 focus-visible:ring-gray-500':
              variant === 'outline',
            'hover:bg-gray-100 focus-visible:ring-gray-500':
              variant === 'ghost',
            'bg-red-600 text-white hover:bg-red-700 focus-visible:ring-red-500':
              variant === 'destructive',
          },
          
          // Sizes
          {
            'h-8 px-3 text-sm': size === 'sm',
            'h-10 px-4 text-base': size === 'md',
            'h-12 px-6 text-lg': size === 'lg',
            'h-10 w-10 p-0': size === 'icon',
          },
          
          className
        )}
        {...props}
      >
        {isLoading ? (
          <Spinner size="sm" />
        ) : leftIcon ? (
          <span className="shrink-0">{leftIcon}</span>
        ) : null}
        {children}
        {rightIcon && !isLoading && (
          <span className="shrink-0">{rightIcon}</span>
        )}
      </button>
    );
  }
);

Button.displayName = 'Button';
```

### 5.2 Container/Presentational Pattern

```typescript
// Presentational component (pure UI)
interface UserCardPresentationProps {
  user: {
    id: string;
    name: string;
    email: string;
    avatar?: string;
  };
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
}

function UserCardPresentation({ user, onEdit, onDelete }: UserCardPresentationProps) {
  return (
    <div className="p-4 border rounded-lg">
      <img src={user.avatar || '/default-avatar.png'} alt="" />
      <h3>{user.name}</h3>
      <p>{user.email}</p>
      <div className="flex gap-2">
        <button onClick={() => onEdit(user.id)}>Edit</button>
        <button onClick={() => onDelete(user.id)}>Delete</button>
      </div>
    </div>
  );
}

// Container component (logic)
interface UserCardContainerProps {
  userId: string;
}

export function UserCard({ userId }: UserCardContainerProps) {
  const { data: user, isLoading, error } = trpc.users.get.useQuery({ id: userId });
  const deleteUser = trpc.users.delete.useMutation({
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] });
    },
  });
  
  if (isLoading) return <Skeleton />;
  if (error) return <ErrorMessage error={error} />;
  if (!user) return null;
  
  return (
    <UserCardPresentation
      user={user}
      onEdit={(id) => router.push(`/users/${id}/edit`)}
      onDelete={(id) => deleteUser.mutate({ id })}
    />
  );
}
```

---

## 6. Accessibility Patterns

### 6.1 ARIA Patterns

```tsx
// Modal with proper ARIA
function Modal({ isOpen, onClose, title, children }: ModalProps) {
  const modalRef = useRef<HTMLDivElement>(null);
  
  // Focus trap
  useEffect(() => {
    if (!isOpen) return;
    
    const focusableElements = modalRef.current?.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    const firstElement = focusableElements?.[0] as HTMLElement;
    const lastElement = focusableElements?.[focusableElements.length - 1] as HTMLElement;
    
    const handleTab = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return;
      
      if (e.shiftKey && document.activeElement === firstElement) {
        e.preventDefault();
        lastElement?.focus();
      } else if (!e.shiftKey && document.activeElement === lastElement) {
        e.preventDefault();
        firstElement?.focus();
      }
    };
    
    document.addEventListener('keydown', handleTab);
    firstElement?.focus();
    
    return () => document.removeEventListener('keydown', handleTab);
  }, [isOpen]);
  
  // Close on Escape
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    
    if (isOpen) {
      document.addEventListener('keydown', handleEscape);
      document.body.style.overflow = 'hidden';
    }
    
    return () => {
      document.removeEventListener('keydown', handleEscape);
      document.body.style.overflow = '';
    };
  }, [isOpen, onClose]);
  
  if (!isOpen) return null;
  
  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      aria-describedby="modal-description"
      ref={modalRef}
    >
      <div
        className="fixed inset-0 bg-black/50"
        onClick={onClose}
        aria-hidden="true"
      />
      <div className="fixed inset-4 md:inset-auto md:top-1/2 md:left-1/2 md:-translate-x-1/2 md:-translate-y-1/2 md:w-full md:max-w-lg bg-white rounded-lg shadow-xl">
        <header className="flex items-center justify-between p-4 border-b">
          <h2 id="modal-title">{title}</h2>
          <button
            onClick={onClose}
            aria-label="Close modal"
          >
            <CloseIcon />
          </button>
        </header>
        <div id="modal-description" className="p-4">
          {children}
        </div>
      </div>
    </div>
  );
}

// Accessible data table
function DataTable<T>({ columns, data }: DataTableProps<T>) {
  return (
    <table
      role="grid"
      aria-label="User data"
      aria-rowcount={data.length}
      aria-colcount={columns.length}
    >
      <thead>
        <tr role="row">
          {columns.map((col, i) => (
            <th
              key={col.key}
              role="columnheader"
              aria-colindex={i + 1}
              scope="col"
            >
              {col.label}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {data.map((row, rowIndex) => (
          <tr key={row.id} role="row" aria-rowindex={rowIndex + 1}>
            {columns.map((col, colIndex) => (
              <td
                key={col.key}
                role="gridcell"
                aria-colindex={colIndex + 1}
              >
                {col.render(row)}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}
```

### 6.2 Keyboard Navigation

```tsx
// Keyboard navigable dropdown
function Dropdown({ options, value, onChange }: DropdownProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [focusedIndex, setFocusedIndex] = useState(-1);
  const containerRef = useRef<HTMLDivElement>(null);
  
  const handleKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'Enter':
      case ' ':
        if (isOpen && focusedIndex >= 0) {
          onChange(options[focusedIndex]);
          setIsOpen(false);
        } else {
          setIsOpen(true);
        }
        break;
        
      case 'ArrowDown':
        e.preventDefault();
        if (!isOpen) {
          setIsOpen(true);
          setFocusedIndex(0);
        } else {
          setFocusedIndex((prev) => Math.min(prev + 1, options.length - 1));
        }
        break;
        
      case 'ArrowUp':
        e.preventDefault();
        setFocusedIndex((prev) => Math.max(prev - 1, 0));
        break;
        
      case 'Escape':
        setIsOpen(false);
        setFocusedIndex(-1);
        break;
        
      case 'Tab':
        setIsOpen(false);
        break;
    }
  };
  
  return (
    <div
      ref={containerRef}
      role="combobox"
      aria-expanded={isOpen}
      aria-haspopup="listbox"
      aria-controls="dropdown-list"
      aria-activedescendant={focusedIndex >= 0 ? `option-${focusedIndex}` : undefined}
      tabIndex={0}
      onKeyDown={handleKeyDown}
      onClick={() => setIsOpen(!isOpen)}
    >
      <span>{value?.label || 'Select...'}</span>
      
      {isOpen && (
        <ul
          id="dropdown-list"
          role="listbox"
          aria-label="Options"
        >
          {options.map((option, index) => (
            <li
              key={option.value}
              id={`option-${index}`}
              role="option"
              aria-selected={value?.value === option.value}
              tabIndex={focusedIndex === index ? 0 : -1}
            >
              {option.label}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
```

---

## 7. Testing Strategy

### 7.1 Test Configuration

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./tests/setup.ts'],
    
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      thresholds: {
        statements: 80,
        branches: 80,
        functions: 80,
        lines: 80,
      },
    },
    
    include: ['**/*.test.{ts,tsx}'],
    exclude: ['node_modules', 'dist'],
    
    // Timeouts
    hookTimeout: 10000,
    testTimeout: 10000,
  },
});
```

### 7.2 Component Tests

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
  
  it('calls onClick when clicked', async () => {
    const handleClick = vi.fn();
    const user = userEvent.setup();
    
    render(<Button onClick={handleClick}>Click me</Button>);
    await user.click(screen.getByRole('button'));
    
    expect(handleClick).toHaveBeenCalledOnce();
  });
  
  it('is disabled when isLoading is true', () => {
    render(<Button isLoading>Loading</Button>);
    expect(screen.getByRole('button')).toBeDisabled();
  });
  
  it('shows loading spinner when isLoading', () => {
    render(<Button isLoading loadingText="Saving...">Save</Button>);
    expect(screen.getByText('Saving...')).toBeInTheDocument();
  });
  
  it('does not call onClick when disabled', async () => {
    const handleClick = vi.fn();
    const user = userEvent.setup();
    
    render(<Button disabled onClick={handleClick}>Disabled</Button>);
    await user.click(screen.getByRole('button'));
    
    expect(handleClick).not.toHaveBeenCalled();
  });
  
  it('focuses correctly with keyboard', async () => {
    const user = userEvent.setup();
    
    render(<Button>Focus me</Button>);
    await user.tab();
    
    expect(screen.getByRole('button')).toHaveFocus();
  });
  
  it('shows icon when leftIcon provided', () => {
    const icon = <span data-testid="icon">🚀</span>;
    render(<Button leftIcon={icon}>Launch</Button>);
    
    expect(screen.getByTestId('icon')).toBeInTheDocument();
  });
});
```

### 7.3 Integration Tests

```typescript
// tests/e2e/auth.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('user can log in with valid credentials', async ({ page }) => {
    await page.goto('/login');
    
    // Fill form
    await page.getByLabel('Email').fill('user@example.com');
    await page.getByLabel('Password').fill('securepassword123');
    
    // Submit
    await page.getByRole('button', { name: 'Log in' }).click();
    
    // Verify redirect to dashboard
    await expect(page).toHaveURL('/dashboard');
    
    // Verify user is logged in
    await expect(page.getByText('Welcome, user@example.com')).toBeVisible();
  });
  
  test('shows error with invalid credentials', async ({ page }) => {
    await page.goto('/login');
    
    await page.getByLabel('Email').fill('wrong@example.com');
    await page.getByLabel('Password').fill('wrongpassword');
    await page.getByRole('button', { name: 'Log in' }).click();
    
    // Verify error message
    await expect(page.getByRole('alert')).toContainText('Invalid email or password');
    
    // Verify still on login page
    await expect(page).toHaveURL('/login');
  });
  
  test('requires authentication for protected routes', async ({ page }) => {
    await page.goto('/dashboard');
    
    // Verify redirect to login
    await expect(page).toHaveURL('/login?redirect=/dashboard');
  });
});
```

---

## 8. Anti-Patterns

| Anti-Pattern | Specific Failure Mode | Prevention |
|--------------|----------------------|------------|
| **Giant bundles** | Slow load, poor Core Vitals | Code splitting, tree shaking |
| **Prop drilling** | Complex maintenance | Context, Zustand, composition |
| **No error boundaries** | White screen of death | ErrorBoundary component |
| **Blocking main thread** | Poor interactivity | Web Workers, virtualization |
| **Memory leaks** | Performance degradation | Cleanup subscriptions, WeakRef |
| **No loading states** | Blank screens | Skeletons, spinners |
| **Layout shift** | Poor CLS | Explicit dimensions |
| **Render-blocking resources** | Slow LCP | async/defer scripts, preload |
| **No accessibility** | Screen reader broken | ARIA, semantic HTML, testing |
| **Over-engineering** | Complex, hard to maintain | YAGNI, start simple |

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/WEB.md` - Web architecture
- `architecture/UI.md` - UI patterns
- `architecture/SECURITY.md` - Frontend security
- `architecture/CLOUD.md` - CDN deployment

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition
- `specs/SECURITY.md` - Security doctrine

### Interface Contracts
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/CONTROL_PLANE.md` - Agent sequencing
- `interfaces/STORE_MODEL.md` - Store semantics

### Methodology
- `methodology/ARCHITECTURE.md` - Architecture methodology
- `methodology/FRONTEND_PERFORMANCE.md` - Frontend performance