# GRAPHQL.md - GraphQL Architecture Standards

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

---

## 1. Schema Design Fundamentals

### 1.1 Schema Structure and Types

```graphql
# Basic scalar types
# String, Int, Float, Boolean, ID

# Custom scalar types for domain-specific data
scalar DateTime
scalar UUID
scalar JSON
scalar URL
scalar EmailAddress
scalar PositiveInt
scalar Markdown

# Enums should have clear naming conventions
enum UserRole {
  USER
  ADMIN
  SUPER_ADMIN
  SERVICE_ACCOUNT
  READ_ONLY
}

enum OrderStatus {
  PENDING
  CONFIRMED
  PROCESSING
  SHIPPED
  DELIVERED
  CANCELLED
  REFUNDED
  ON_HOLD
}

enum ProductCategory {
  ELECTRONICS
  CLOTHING
  HOME_AND_GARDEN
  SPORTS
  BOOKS
  TOYS
  FOOD
  BEAUTY
  AUTO
  INDUSTRIAL
}

# Interfaces for polymorphic types
interface Node {
  id: ID!
}

interface Timestamped {
  createdAt: DateTime!
  updatedAt: DateTime!
}

interface UserGeneratable {
  createdBy: User
  updatedBy: User
}
```

### 1.2 Object Types and Fields

```graphql
# Complete user type definition
type User implements Node & Timestamped {
  # Primary identifiers
  id: ID!
  email: String!
  externalId: String
  
  # Profile information
  displayName: String!
  firstName: String
  lastName: String
  avatarUrl: URL
  bio: String
  
  # Status and role
  role: UserRole!
  status: UserStatus!
  emailVerified: Boolean!
  accountLocked: Boolean!
  
  # Timestamps
  createdAt: DateTime!
  updatedAt: DateTime!
  lastLoginAt: DateTime
  
  # Relationships
  manager: User
  team: Team
  permissions: [Permission!]!
  preferences: UserPreferences!
  
  # Computed fields
  fullName: String!
  initials: String!
  isActive: Boolean!
  
  # Connections (for pagination)
  teams: TeamConnection!
  orders(first: Int, after: String): OrderConnection!
  notifications(unreadOnly: Boolean): NotificationConnection!
}

type UserPreferences {
  theme: Theme!
  language: String!
  timezone: String!
  notificationsEnabled: Boolean!
  emailNotifications: EmailNotificationPreferences!
  privacySettings: PrivacySettings!
}

type Team implements Node & Timestamped {
  id: ID!
  name: String!
  description: String
  avatarUrl: URL
  createdAt: DateTime!
  updatedAt: DateTime!
  
  members(first: Int, after: String): TeamMemberConnection!
  projects(first: Int, after: String): ProjectConnection!
  owner: User!
}

type Product implements Node & Timestamped {
  id: ID!
  sku: String!
  name: String!
  slug: String!
  description: String!
  category: ProductCategory!
  
  # Pricing
  price: Money!
  compareAtPrice: Money
  costPrice: Money
  
  # Media
  images: [ProductImage!]!
  primaryImage: ProductImage
  thumbnailUrl: URL
  
  # Inventory
  inventory: InventoryStatus!
  availableForSale: Boolean!
  
  # Attributes
  attributes: [ProductAttribute!]!
  specifications: [Specification!]!
  tags: [String!]!
  
  # Variants
  variants: [ProductVariant!]!
  hasVariants: Boolean!
  
  # Review stats
  averageRating: Float
  reviewCount: Int!
  
  # Status
  status: ProductStatus!
  publishedAt: DateTime
  
  # SEO
  seoTitle: String
  seoDescription: String
  meta: ProductMeta!
}

type Money {
  amount: Float!
  currency: Currency!
  formatted: String!
}

type InventoryStatus {
  available: Int!
  reserved: Int!
  total: Int!
  lowStockThreshold: Int
  isLowStock: Boolean!
  warehouseLocation: String
}

type ProductVariant {
  id: ID!
  name: String!
  sku: String!
  attributes: [VariantAttribute!]!
  price: Money
  compareAtPrice: Money
  inventory: Int!
  availableForSale: Boolean!
  image: ProductImage
}

type Order implements Node & Timestamped {
  id: ID!
  orderNumber: String!
  status: OrderStatus!
  
  # Customer
  customer: User!
  billingAddress: Address!
  shippingAddress: Address!
  
  # Items
  items: [OrderItem!]!
  itemCount: Int!
  subtotal: Money!
  
  # Totals
  taxTotal: Money!
  shippingTotal: Money!
  discountTotal: Money!
  total: Money!
  
  # Payment
  paymentStatus: PaymentStatus!
  paymentMethod: PaymentMethod
  transactions: [PaymentTransaction!]!
  
  # Fulfillment
  fulfillmentStatus: FulfillmentStatus!
  trackingNumber: String
  trackingUrl: URL
  
  # Events
  events: [OrderEvent!]!
  
  # Timestamps
  placedAt: DateTime
  confirmedAt: DateTime
  shippedAt: DateTime
  deliveredAt: DateTime
  cancelledAt: DateTime
}

# Union types for polymorphic queries
union SearchResult = Product | Category | Brand | ContentPage

union PaymentIntent = CreditCardPayment | BankTransferPayment | CryptoPayment

union ContentBlock = TextBlock | ImageBlock | VideoBlock | EmbedBlock

# Input types for mutations
input CreateUserInput {
  email: String!
  password: String!
  displayName: String!
  firstName: String
  lastName: String
  role: UserRole = USER
  attributes: JSON
}

input UpdateUserInput {
  email: String
  displayName: String
  firstName: String
  lastName: String
  avatarUrl: URL
  bio: String
  preferences: UserPreferencesInput
}

input UserPreferencesInput {
  theme: Theme
  language: String
  timezone: String
  notificationsEnabled: Boolean
}

input AddressInput {
  recipientName: String!
  addressLine1: String!
  addressLine2: String
  city: String!
  state: String!
  postalCode: String!
  country: String!
  phoneNumber: String
  instructions: String
}

input OrderItemInput {
  productId: ID!
  variantId: ID
  quantity: Int!
  customAttributes: JSON
}

input ProductFilterInput {
  category: ProductCategory
  categories: [ProductCategory!]
  priceRange: PriceRangeInput
  inStock: Boolean
  onSale: Boolean
  tags: [String!]
  searchQuery: String
  minRating: Float
}

input PriceRangeInput {
  min: Float
  max: Float
}
```

## 2. Complete Schema Examples

### 2.1 E-Commerce GraphQL Schema

```graphql
# schema.graphql - Complete e-commerce GraphQL schema

schema {
  query: Query
  mutation: Mutation
  subscription: Subscription
}

# Scalars
scalar DateTime
scalar UUID
scalar JSON
scalar URL
scalar EmailAddress
scalar PositiveInt
scalar Markdown
scalar Decimal

scalar Upload

# Enums
enum UserRole {
  USER
  ADMIN
  SUPER_ADMIN
  SERVICE_ACCOUNT
  READ_ONLY
}

enum UserStatus {
  ACTIVE
  INACTIVE
  SUSPENDED
  DELETED
  PENDING_VERIFICATION
}

enum OrderStatus {
  PENDING
  AWAITING_PAYMENT
  CONFIRMED
  PROCESSING
  SHIPPED
  OUT_FOR_DELIVERY
  DELIVERED
  CANCELLED
  REFUNDED
  ON_HOLD
}

enum PaymentStatus {
  PENDING
  PROCESSING
  AUTHORIZED
  CAPTURED
  FAILED
  REFUNDED
  PARTIALLY_REFUNDED
}

enum FulfillmentStatus {
  UNFULFILLED
  PARTIALLY_FULFILLED
  FULFILLED
  CANCELLED
}

enum ProductStatus {
  DRAFT
  ACTIVE
  INACTIVE
  DISCONTINUED
  ARCHIVED
}

enum InventoryAlertLevel {
  NONE
  LOW
  CRITICAL
}

enum Theme {
  LIGHT
  DARK
  SYSTEM
}

# Interfaces
interface Node {
  id: ID!
}

interface Timestamped {
  createdAt: DateTime!
  updatedAt: DateTime!
}

interface PaginatedConnection {
  pageInfo: PageInfo!
  totalCount: Int!
}

# Types
type Query {
  # User queries
  me: User
  user(id: ID!): User
  users(
    filter: UserFilterInput
    sort: [UserSortInput!]
    pagination: PaginationInput
  ): UserConnection!
  searchUsers(query: String!, limit: Int = 10): [User!]!
  
  # Product queries
  product(id: ID, slug: String): Product
  products(
    filter: ProductFilterInput
    sort: [ProductSortInput!]
    pagination: PaginationInput
  ): ProductConnection!
  featuredProducts(limit: Int = 10): [Product!]!
  productRecommendations(productId: ID!): [Product!]!
  
  # Order queries
  order(id: ID!): Order
  orders(
    filter: OrderFilterInput
    sort: [OrderSortInput!]
    pagination: PaginationInput
  ): OrderConnection!
  myOrders(
    filter: OrderFilterInput
    pagination: PaginationInput
  ): OrderConnection!
  
  # Cart queries
  cart(id: ID!): Cart
  myCart: Cart!
  
  # Category queries
  category(id: ID, slug: String): Category
  categories(parentId: ID, depth: Int = 2): [Category!]!
  categoryTree(depth: Int = 3): [Category!]!
  
  # Search
  search(query: String!, filters: SearchFiltersInput, pagination: PaginationInput): SearchResults!
  
  # Checkout
  checkout(token: String!): Checkout
  paymentIntent(clientSecret: String!): PaymentIntent
  
  # Admin queries
  adminStats(startDate: DateTime!, endDate: DateTime!): AdminStats!
  adminDashboard: AdminDashboard!
}

type Mutation {
  # Auth mutations
  register(input: RegisterInput!): AuthPayload!
  login(email: EmailAddress!, password: String!): AuthPayload!
  logout: Boolean!
  refreshToken(token: String!): AuthPayload!
  verifyEmail(token: String!): Boolean!
  requestPasswordReset(email: EmailAddress!): Boolean!
  resetPassword(token: String!, newPassword: String!): Boolean!
  
  # User mutations
  createUser(input: CreateUserInput!): User!
  updateUser(id: ID!, input: UpdateUserInput!): User!
  deleteUser(id: ID!): Boolean!
  changeUserRole(id: ID!, role: UserRole!): User!
  suspendUser(id: ID!, reason: String): User!
  
  # Product mutations
  createProduct(input: CreateProductInput!): Product!
  updateProduct(id: ID!, input: UpdateProductInput!): Product!
  deleteProduct(id: ID!): Boolean!
  publishProduct(id: ID!): Product!
  unpublishProduct(id: ID!): Product!
  
  # Cart mutations
  addToCart(productId: ID!, variantId: ID, quantity: Int!): Cart!
  updateCartItem(itemId: ID!, quantity: Int!): Cart!
  removeFromCart(itemId: ID!): Cart!
  clearCart: Cart!
  applyCoupon(code: String!): Cart!
  removeCoupon: Cart!
  
  # Order mutations
  createOrder(input: CreateOrderInput!): Order!
  cancelOrder(id: ID!, reason: String): Order!
  updateOrderStatus(id: ID!, status: OrderStatus!, comment: String): Order!
  addOrderNote(id: ID!, note: String!): Order!
  
  # Payment mutations
  initializePayment(input: PaymentInput!): PaymentIntent!
  confirmPayment(intentId: String!): PaymentResult!
  refundPayment(paymentId: ID!, amount: Decimal, reason: String): RefundResult!
  
  # File uploads
  uploadFile(input: UploadInput!): FileUpload!
  deleteFile(id: ID!): Boolean!
}

type Subscription {
  # Order subscriptions
  orderStatusChanged(orderId: ID!): OrderStatusEvent!
  myOrdersUpdated: Order!
  
  # Product subscriptions
  productUpdated(productId: ID!): Product!
  productInventoryChanged(productIds: [ID!]!): ProductInventoryUpdate!
  
  # Cart subscriptions
  cartUpdated: Cart!
  
  # Notification subscriptions
  notificationReceived: Notification!
  
  # Chat subscriptions
  messageReceived(threadId: ID!): Message!
}

# Connection Types
type UserConnection implements PaginatedConnection {
  edges: [UserEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type UserEdge {
  node: User!
  cursor: String!
}

type ProductConnection implements PaginatedConnection {
  edges: [ProductEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type ProductEdge {
  node: Product!
  cursor: String!
}

type OrderConnection implements PaginatedConnection {
  edges: [OrderEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type OrderEdge {
  node: Order!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

# Object Types
type User implements Node & Timestamped {
  id: ID!
  email: String!
  displayName: String!
  firstName: String
  lastName: String
  avatarUrl: URL
  bio: String
  role: UserRole!
  status: UserStatus!
  emailVerified: Boolean!
  accountLocked: Boolean!
  createdAt: DateTime!
  updatedAt: DateTime!
  lastLoginAt: DateTime
  team: Team
  manager: User
  preferences: UserPreferences!
  
  # Computed
  fullName: String!
  initials: String!
  isActive: Boolean!
  
  # Relationships
  orders(filter: OrderFilterInput, pagination: PaginationInput): OrderConnection!
  teams: [Team!]!
}

type Team implements Node & Timestamped {
  id: ID!
  name: String!
  description: String
  avatarUrl: URL
  createdAt: DateTime!
  updatedAt: DateTime!
  owner: User!
  members(first: Int, after: String): TeamMemberConnection!
  projects(first: Int, after: String): ProjectConnection!
}

type TeamMemberConnection implements PaginatedConnection {
  edges: [TeamMemberEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type TeamMemberEdge {
  node: TeamMember!
  cursor: String!
}

type TeamMember {
  user: User!
  role: TeamRole!
  joinedAt: DateTime!
}

type Product implements Node & Timestamped {
  id: ID!
  sku: String!
  name: String!
  slug: String!
  description: String!
  descriptionHtml: String!
  category: Category!
  categoryPath: [Category!]!
  brand: Brand
  
  # Pricing
  price: Money!
  compareAtPrice: Money
  costPrice: Money
  margin: Money
  marginPercent: Float
  onSale: Boolean!
  discountPercent: Int
  
  # Media
  images: [ProductImage!]!
  primaryImage: ProductImage
  thumbnailUrl: URL
  videoUrl: URL
  
  # Inventory
  inventory: InventoryStatus!
  availableForSale: Boolean!
  trackInventory: Boolean!
  
  # Attributes
  attributes: [ProductAttribute!]!
  specifications: [Specification!]!
  tags: [String!]!
  
  # Variants
  hasVariants: Boolean!
  variants: [ProductVariant!]!
  options: [ProductOption!]!
  
  # Reviews
  reviews(first: Int, after: String): ReviewConnection!
  averageRating: Float
  reviewCount: Int!
  
  # SEO
  seoTitle: String
  seoDescription: String
  meta: ProductMeta!
  
  # Status
  status: ProductStatus!
  publishedAt: DateTime
  
  # Timestamps
  createdAt: DateTime!
  updatedAt: DateTime!
  
  # Related
  relatedProducts: [Product!]!
  crossSellProducts: [Product!]!
}

type ProductVariant {
  id: ID!
  name: String!
  sku: String!
  price: Money!
  compareAtPrice: Money
  inventory: Int!
  availableForSale: Boolean!
  weight: Float
  weightUnit: String
  image: ProductImage
  attributes: [VariantAttribute!]!
  selectedOptions: [SelectedOption!]!
}

type ProductOption {
  id: ID!
  name: String!
  values: [String!]!
}

type SelectedOption {
  name: String!
  value: String!
}

type VariantAttribute {
  name: String!
  value: String!
}

type ProductAttribute {
  name: String!
  value: String!
  displayValue: String
}

type Specification {
  name: String!
  value: String!
}

type ProductImage {
  id: ID!
  url: URL!
  altText: String
  width: Int
  height: Int
  sortOrder: Int!
  isPrimary: Boolean!
}

type ProductMeta {
  title: String
  description: String
  keywords: [String!]!
  canonicalUrl: URL
  image: ProductImage
  schema: JSON
}

type Category implements Node {
  id: ID!
  name: String!
  slug: String!
  description: String
  image: ProductImage
  parent: Category
  children: [Category!]!
  productCount: Int!
  products(first: Int, after: String): ProductConnection!
}

type Brand implements Node {
  id: ID!
  name: String!
  slug: String!
  description: String
  logoUrl: URL
  website: URL
  products(first: Int, after: String): ProductConnection!
}

type InventoryStatus {
  available: Int!
  reserved: Int!
  total: Int!
  lowStockThreshold: Int
  isLowStock: Boolean!
  alertLevel: InventoryAlertLevel!
  warehouseLocation: String
  nextRestockDate: DateTime
}

type Review implements Node & Timestamped {
  id: ID!
  product: Product!
  author: User!
  rating: Int!
  title: String
  content: String!
  pros: [String!]
  cons: [String!]
  images: [ReviewImage!]!
  verified: Boolean!
  helpfulCount: Int!
  status: ReviewStatus!
  createdAt: DateTime!
  updatedAt: DateTime!
}

type ReviewImage {
  id: ID!
  url: URL!
  altText: String
}

type Order implements Node & Timestamped {
  id: ID!
  orderNumber: String!
  status: OrderStatus!
  
  # Customer
  customer: User!
  billingAddress: Address!
  shippingAddress: Address!
  
  # Items
  items: [OrderItem!]!
  itemCount: Int!
  
  # Totals
  subtotal: Money!
  taxTotal: Money!
  shippingTotal: Money!
  discountTotal: Money!
  total: Money!
  
  # Payment
  paymentStatus: PaymentStatus!
  paymentMethod: PaymentMethod
  transactions: [PaymentTransaction!]!
  
  # Fulfillment
  fulfillmentStatus: FulfillmentStatus!
  trackingNumber: String
  trackingUrl: URL
  carrier: String
  
  # Notes
  notes: [OrderNote!]!
  
  # Timestamps
  createdAt: DateTime!
  updatedAt: DateTime!
  placedAt: DateTime
  confirmedAt: DateTime
  shippedAt: DateTime
  deliveredAt: DateTime
  cancelledAt: DateTime
  refundRequestedAt: DateTime
  refundProcessedAt: DateTime
  
  # Events
  events: [OrderEvent!]!
}

type OrderItem {
  id: ID!
  product: Product!
  variant: ProductVariant
  name: String!
  sku: String!
  quantity: Int!
  unitPrice: Money!
  totalPrice: Money!
  attributes: [SelectedOption!]!
  image: ProductImage
  canCancel: Boolean!
  canReturn: Boolean!
}

type OrderNote {
  id: ID!
  content: String!
  author: User!
  createdAt: DateTime!
  isInternal: Boolean!
}

type OrderEvent {
  id: ID!
  type: String!
  status: OrderStatus
  comment: String
  metadata: JSON
  actor: User
  createdAt: DateTime!
}

type PaymentMethod {
  id: ID!
  type: PaymentMethodType!
  lastFourDigits: String
  cardBrand: String
  expiryMonth: Int
  expiryYear: Int
  bankName: String
  isDefault: Boolean!
}

type PaymentTransaction {
  id: ID!
  type: TransactionType!
  amount: Money!
  status: TransactionStatus!
  gateway: String!
  gatewayTransactionId: String
  gatewayResponse: JSON
  createdAt: DateTime!
  error: String
}

type Address {
  id: ID!
  recipientName: String!
  addressLine1: String!
  addressLine2: String
  city: String!
  state: String!
  postalCode: String!
  country: String!
  countryCode: String!
  phoneNumber: String
  instructions: String
  isDefault: Boolean!
  label: String
}

type Cart implements Node {
  id: ID!
  customer: User
  sessionId: String
  items: [CartItem!]!
  itemCount: Int!
  quantityCount: Int!
  
  # Pricing
  subtotal: Money!
  taxTotal: Money
  shippingTotal: Money
  discountTotal: Money!
  total: Money!
  
  # Discounts
  discountCodes: [DiscountCode!]!
  appliedDiscounts: [AppliedDiscount!]!
  
  # Shipping
  availableShippingMethods: [ShippingMethod!]!
  shippingAddress: Address
  shippingMethod: ShippingMethod
  
  # Coupon
  couponCode: String
  couponDiscount: Money
  
  # Validation
  validationErrors: [CartValidationError!]!
  isValid: Boolean!
  
  # Timestamps
  createdAt: DateTime!
  updatedAt: DateTime!
  expiresAt: DateTime
}

type CartItem {
  id: ID!
  product: Product!
  variant: ProductVariant
  quantity: Int!
  unitPrice: Money!
  totalPrice: Money!
  attributes: [SelectedOption!]!
  image: ProductImage
  maxQuantity: Int!
  availableForSale: Boolean!
  validationErrors: [String!]!
}

type Money {
  amount: Float!
  currency: Currency!
  symbol: String!
  formatted: String!
}

type Currency {
  code: String!
  symbol: String!
  name: String!
  exchangeRate: Float
}

type DiscountCode {
  id: ID!
  code: String!
  type: DiscountType!
  value: Float!
  minimumCartValue: Money
  maximumDiscount: Money
  usageLimit: Int
  usedCount: Int!
  validFrom: DateTime
  validUntil: DateTime
  isValid: Boolean!
}

type AppliedDiscount {
  code: String!
  type: DiscountType!
  value: Float!
  amount: Money!
}

type ShippingMethod {
  id: ID!
  name: String!
  description: String
  price: Money!
  estimatedDeliveryDays: Int
  carrier: String
}

type CartValidationError {
  type: CartValidationErrorType!
  message: String!
  field: String
  code: String
}

type Checkout implements Node {
  id: ID!
  cart: Cart!
  step: CheckoutStep!
  completedSteps: [CheckoutStep!]!
  
  # Contact
  email: String!
  
  # Addresses
  shippingAddress: Address
  billingAddress: Address
  billingAddressSameAsShipping: Boolean!
  
  # Shipping
  shippingMethod: ShippingMethod
  
  # Payment
  paymentMethod: PaymentMethod
  paymentIntent: PaymentIntent
  
  # Discounts
  discountCodes: [String!]!
  
  # Order
  order: Order
  orderId: ID
  
  # Timestamps
  expiresAt: DateTime
}

type PaymentIntent {
  id: ID!
  clientSecret: String!
  amount: Money!
  status: PaymentIntentStatus!
  paymentMethod: PaymentMethod
  gateway: String!
  returnUrl: URL!
  metadata: JSON
}

type UserPreferences {
  theme: Theme!
  language: String!
  timezone: String!
  dateFormat: String!
  numberFormat: String!
  weightUnit: String!
  distanceUnit: String!
  notificationsEnabled: Boolean!
  emailNotifications: EmailNotificationPreferences!
  privacySettings: PrivacySettings!
}

type EmailNotificationPreferences {
  marketing: Boolean!
  orderUpdates: Boolean!
  priceAlerts: Boolean!
  newsletter: Boolean!
  productUpdates: Boolean!
}

type PrivacySettings {
  profileVisibility: ProfileVisibility!
  showEmail: Boolean!
  showOrders: Boolean!
}

# Auth Types
type AuthPayload {
  token: String!
  refreshToken: String!
  expiresAt: DateTime!
  user: User!
}

type Notification implements Node & Timestamped {
  id: ID!
  type: NotificationType!
  title: String!
  body: String!
  data: JSON
  readAt: DateTime
  isRead: Boolean!
  actionUrl: URL
  createdAt: DateTime!
}

type Message implements Node {
  id: ID!
  thread: MessageThread!
  author: User!
  content: String!
  contentHtml: String!
  attachments: [MessageAttachment!]!
  createdAt: DateTime!
  editedAt: DateTime
  isEdited: Boolean!
}

type MessageThread implements Node {
  id: ID!
  participants: [User!]!
  messages(first: Int, after: String): MessageConnection!
  lastMessage: Message!
  unreadCount: Int!
  createdAt: DateTime!
  updatedAt: DateTime!
}

type MessageAttachment {
  id: ID!
  type: AttachmentType!
  url: URL!
  name: String!
  size: Int!
  mimeType: String!
}

# Admin Types
type AdminStats {
  revenue: RevenueStats!
  orders: OrderStats!
  customers: CustomerStats!
  products: ProductStats!
  traffic: TrafficStats!
}

type RevenueStats {
  total: Money!
  averageOrderValue: Money!
  totalOrders: Int!
  totalRefunds: Money!
  netRevenue: Money!
  revenueByDay: [DailyRevenue!]!
  revenueByCategory: [CategoryRevenue!]!
  topProducts: [ProductRevenue!]!
}

type DailyRevenue {
  date: DateTime!
  revenue: Money!
  orders: Int!
}

type CategoryRevenue {
  category: Category!
  revenue: Money!
  orders: Int!
}

type ProductRevenue {
  product: Product!
  revenue: Money!
  unitsSold: Int!
}

type OrderStats {
  total: Int!
  pending: Int!
  processing: Int!
  shipped: Int!
  delivered: Int!
  cancelled: Int!
  averageDeliveryDays: Float
}

type CustomerStats {
  total: Int!
  newThisMonth: Int!
  active: Int!
  inactive: Int!
  topCustomers: [CustomerStats!]!
}

type CustomerStats {
  customer: User!
  totalOrders: Int!
  totalSpent: Money!
  averageOrderValue: Money!
}

type ProductStats {
  total: Int!
  active: Int!
  outOfStock: Int!
  lowStock: Int!
  totalInventoryValue: Money!
}

type TrafficStats {
  visitors: Int!
  pageViews: Int!
  conversionRate: Float!
  topPages: [PageStats!]!
  topReferrers: [ReferrerStats!]!
}

type PageStats {
  path: String!
  views: Int!
  uniqueViews: Int!
  avgTimeOnPage: Float!
}

type ReferrerStats {
  source: String!
  visitors: Int!
  conversions: Int!
}

type AdminDashboard {
  stats: AdminStats!
  recentOrders: [Order!]!
  lowStockProducts: [Product!]!
  recentReviews: [Review!]!
  alerts: [AdminAlert!]!
}

type AdminAlert {
  id: ID!
  type: AlertType!
  severity: AlertSeverity!
  title: String!
  message: String!
  actionUrl: URL
  createdAt: DateTime!
}

# Input Types
input RegisterInput {
  email: String!
  password: String!
  displayName: String!
  firstName: String
  lastName: String
  marketingConsent: Boolean = false
}

input UserFilterInput {
  role: UserRole
  status: UserStatus
  search: String
  teamId: ID
  createdAfter: DateTime
  createdBefore: DateTime
}

input UserSortInput {
  field: UserSortField!
  direction: SortDirection = ASC
}

enum UserSortField {
  CREATED_AT
  UPDATED_AT
  DISPLAY_NAME
  EMAIL
}

input PaginationInput {
  first: Int
  after: String
  last: Int
  before: String
}

enum SortDirection {
  ASC
  DESC
}

input ProductFilterInput {
  category: ID
  categories: [ID!]
  brand: ID
  brands: [ID!]
  priceRange: PriceRangeInput
  inStock: Boolean
  onSale: Boolean
  tags: [String!]
  status: ProductStatus
  minRating: Float
  search: String
}

input ProductSortInput {
  field: ProductSortField!
  direction: SortDirection = ASC
}

enum ProductSortField {
  CREATED_AT
  UPDATED_AT
  NAME
  PRICE
  BEST_SELLING
  RATING
  RELEVANCE
}

input OrderFilterInput {
  status: OrderStatus
  statuses: [OrderStatus!]
  paymentStatus: PaymentStatus
  fulfillmentStatus: FulfillmentStatus
  createdAfter: DateTime
  createdBefore: DateTime
}

input OrderSortInput {
  field: OrderSortField!
  direction: SortDirection = DESC
}

enum OrderSortField {
  CREATED_AT
  UPDATED_AT
  TOTAL
}

input CreateProductInput {
  name: String!
  description: String!
  categoryId: ID!
  brandId: ID
  sku: String!
  price: Decimal!
  compareAtPrice: Decimal
  costPrice: Decimal
  inventory: Int
  trackInventory: Boolean = true
  status: ProductStatus = DRAFT
  tagIds: [ID!]
  images: [ProductImageInput!]
  variants: [ProductVariantInput!]
  attributes: [ProductAttributeInput!]
  specifications: [SpecificationInput!]
  seo: SEOInput
}

input ProductImageInput {
  url: URL!
  altText: String
  sortOrder: Int
  isPrimary: Boolean = false
}

input ProductVariantInput {
  name: String!
  sku: String!
  price: Decimal
  inventory: Int!
  options: [SelectedOptionInput!]!
  imageUrl: URL
}

input SelectedOptionInput {
  name: String!
  value: String!
}

input ProductAttributeInput {
  name: String!
  value: String!
}

input SpecificationInput {
  name: String!
  value: String!
}

input SEOInput {
  title: String
  description: String
  keywords: [String!]
}

input CreateOrderInput {
  items: [OrderItemInput!]!
  shippingAddressId: ID!
  billingAddressId: ID
  paymentMethodId: ID
  discountCodes: [String!]
  note: String
}

input OrderItemInput {
  productId: ID!
  variantId: ID
  quantity: Int!
}

input PaymentInput {
  paymentMethodId: ID
  gateway: PaymentGateway!
  redirectUrl: URL!
}

enum PaymentGateway {
  STRIPE
  PAYPAL
  SQUARE
  BRAINTREE
}

input UploadInput {
  file: Upload!
  folder: String
  type: FileType!
}

enum FileType {
  PRODUCT_IMAGE
  BRAND_LOGO
  CATEGORY_IMAGE
  USER_AVATAR
  REVIEW_IMAGE
  DOCUMENT
}

input SearchFiltersInput {
  categories: [ID!]
  priceRange: PriceRangeInput
  brands: [ID!]
  rating: Int
  inStock: Boolean
  onSale: Boolean
}

type SearchResults {
  products(first: Int, after: String): ProductConnection!
  categories: [Category!]!
  brands: [Brand!]!
  content: [ContentPage!]!
  totalResults: Int!
  facets: [SearchFacet!]!
}

type SearchFacet {
  name: String!
  values: [FacetValue!]!
}

type FacetValue {
  value: String!
  count: Int!
  selected: Boolean!
}

type ContentPage {
  id: ID!
  title: String!
  slug: String!
  excerpt: String
}
```

## 3. Resolver Strategies

### 3.1 Resolver Pattern Implementations

```typescript
// resolvers/user.resolver.ts - Comprehensive user resolvers

import { 
  GraphQLFieldResolver, 
  GraphQLScalarType,
  Kind 
} from 'graphql';
import { DataLoader } from './dataloader';
import { AuthorizationService } from './auth.service';
import { Logger } from './logger';

const dataloader = new DataLoader();
const auth = new AuthorizationService();
const logger = new Logger();

// Scalar resolvers
const UUIDScalar: GraphQLScalarType = new GraphQLScalarType({
  name: 'UUID',
  description: 'UUID custom scalar type',
  serialize(value: unknown): string {
    if (typeof value !== 'string') {
      throw new Error('UUID must be a string');
    }
    return value;
  },
  parseValue(value: unknown): string {
    if (typeof value !== 'string') {
      throw new Error('UUID must be a string');
    }
    if (!isValidUUID(value)) {
      throw new Error('Invalid UUID format');
    }
    return value;
  },
  parseLiteral(ast): string | null {
    if (ast.kind === Kind.STRING) {
      if (!isValidUUID(ast.value)) {
        throw new Error('Invalid UUID format');
      }
      return ast.value;
    }
    return null;
  },
});

const DateTimeScalar: GraphQLScalarType = new GraphQLScalarType({
  name: 'DateTime',
  description: 'ISO 8601 DateTime',
  serialize(value: unknown): string {
    if (value instanceof Date) {
      return value.toISOString();
    }
    if (typeof value === 'string') {
      return value;
    }
    throw new Error('DateTime must be a Date or ISO string');
  },
  parseValue(value: unknown): Date {
    if (typeof value === 'string') {
      return new Date(value);
    }
    throw new Error('DateTime must be an ISO string');
  },
  parseLiteral(ast): Date | null {
    if (ast.kind === Kind.STRING) {
      return new Date(ast.value);
    }
    return null;
  },
});

// Field resolvers with DataLoader batching
const userResolvers = {
  Query: {
    me: async (_: unknown, __: unknown, context: Context): Promise<User> => {
      if (!context.user) {
        throw new AuthError('Not authenticated');
      }
      return context.user;
    },

    user: async (_: unknown, { id }: { id: string }): Promise<User | null> => {
      return dataloader.loadUser(id);
    },

    users: async (
      _: unknown,
      { filter, sort, pagination }: ListUsersArgs
    ): Promise<Connection<User>> => {
      // Verify admin access
      await auth.requireRole('ADMIN');
      
      const users = await UserService.list({
        filter,
        sort,
        pagination,
      });
      
      return users;
    },

    searchUsers: async (
      _: unknown,
      { query, limit }: { query: string; limit: number }
    ): Promise<User[]> => {
      return UserService.search(query, limit);
    },
  },

  Mutation: {
    createUser: async (
      _: unknown,
      { input }: { input: CreateUserInput },
      context: Context
    ): Promise<User> => {
      await auth.requireRole('ADMIN');
      
      const user = await UserService.create(input);
      
      logger.info(`User created: ${user.id}`, {
        createdBy: context.user?.id,
        email: user.email,
      });
      
      return user;
    },

    updateUser: async (
      _: unknown,
      { id, input }: { id: string; input: UpdateUserInput },
      context: Context
    ): Promise<User> => {
      // Either admin or self
      await auth.requireAnyRole('ADMIN');
      if (context.user?.id !== id) {
        await auth.requireRole('ADMIN');
      }
      
      const user = await UserService.update(id, input);
      
      logger.info(`User updated: ${id}`, {
        updatedBy: context.user?.id,
        fields: Object.keys(input),
      });
      
      return user;
    },

    deleteUser: async (
      _: unknown,
      { id }: { id: string },
      context: Context
    ): Promise<boolean> => {
      await auth.requireRole('ADMIN');
      
      await UserService.delete(id);
      
      logger.info(`User deleted: ${id}`, {
        deletedBy: context.user?.id,
      });
      
      return true;
    },
  },

  Subscription: {
    userUpdated: {
      subscribe: async function* (
        _: unknown,
        { userId }: { userId: string }
      ) {
        for await (const update of UserService.subscribeToUpdates(userId)) {
          yield { userUpdated: update };
        }
      },
    },
  },

  User: {
    // Field resolvers - these batch automatically with DataLoader
    id: (parent: User): string => parent.id,
    email: (parent: User): string => parent.email,
    displayName: (parent: User): string => parent.displayName,
    firstName: (parent: User): string | undefined => parent.firstName,
    lastName: (parent: User): string | undefined => parent.lastName,
    avatarUrl: (parent: User): URL | undefined => parent.avatarUrl,
    bio: (parent: User): string | undefined => parent.bio,
    role: (parent: User): UserRole => parent.role,
    status: (parent: User): UserStatus => parent.status,
    emailVerified: (parent: User): boolean => parent.emailVerified,
    accountLocked: (parent: User): boolean => parent.accountLocked,
    createdAt: (parent: User): DateTime => parent.createdAt,
    updatedAt: (parent: User): DateTime => parent.updatedAt,
    lastLoginAt: (parent: User): DateTime | undefined => parent.lastLoginAt,
    
    // Computed fields
    fullName: (parent: User): string => {
      if (parent.firstName && parent.lastName) {
        return `${parent.firstName} ${parent.lastName}`;
      }
      return parent.displayName;
    },
    
    initials: (parent: User): string => {
      const parts: string[] = [];
      if (parent.firstName) parts.push(parent.firstName[0]);
      if (parent.lastName) parts.push(parent.lastName[0]);
      return parts.join('').toUpperCase() || parent.displayName.slice(0, 2).toUpperCase();
    },
    
    isActive: (parent: User): boolean => {
      return parent.status === 'ACTIVE' && !parent.accountLocked;
    },
    
    // Relationship resolvers with batching
    manager: (parent: User): Promise<User | null> => {
      if (!parent.managerId) return null;
      return dataloader.loadUser(parent.managerId);
    },
    
    team: (parent: User): Promise<Team | null> => {
      if (!parent.teamId) return null;
      return dataloader.loadTeam(parent.teamId);
    },
    
    permissions: async (parent: User): Promise<Permission[]> => {
      return dataloader.loadUserPermissions(parent.id);
    },
    
    preferences: (parent: User): UserPreferences => {
      return parent.preferences;
    },
    
    // Connection resolvers for pagination
    orders: async (
      parent: User,
      { first = 10, after }: ConnectionArgs,
      context: Context
    ): Promise<OrderConnection> => {
      // If not self or admin, don't expose orders
      if (context.user?.id !== parent.id && !auth.hasRole('ADMIN')) {
        throw new AuthError('Not authorized to view orders');
      }
      
      return OrderService.listByUser(parent.id, { first, after });
    },
    
    teams: async (parent: User): Promise<Team[]> => {
      return dataloader.loadUserTeams(parent.id);
    },
    
    notifications: async (
      parent: User,
      { unreadOnly = false }: { unreadOnly?: boolean }
    ): Promise<Notification[]> => {
      return NotificationService.listForUser(parent.id, { unreadOnly });
    },
  },
};

// Pagination helper
interface ConnectionArgs {
  first?: number;
  after?: string;
  last?: number;
  before?: string;
}

async function resolveConnection<T>(
  total: number,
  items: T[],
  { first, after }: ConnectionArgs,
  encodeCursor: (item: T, index: number) => string
): Promise<Connection<T>> {
  const startIndex = after ? decodeCursor(after) + 1 : 0;
  const slicedItems = items.slice(startIndex, startIndex + (first || 10));
  const hasNextPage = startIndex + slicedItems.length < total;
  const hasPreviousPage = startIndex > 0;
  
  return {
    edges: slicedItems.map((item, index) => ({
      node: item,
      cursor: encodeCursor(item, startIndex + index),
    })),
    pageInfo: {
      hasNextPage,
      hasPreviousPage,
      startCursor: slicedItems.length > 0 ? encodeCursor(slicedItems[0], startIndex) : null,
      endCursor: slicedItems.length > 0 ? encodeCursor(slicedItems[slicedItems.length - 1], startIndex + slicedItems.length - 1) : null,
    },
    totalCount: total,
  };
}
```

### 3.2 DataLoader Implementation for N+1 Prevention

```typescript
// dataloader.ts - Complete DataLoader implementation

import DataLoader from 'dataloader';
import { UserService, TeamService, OrderService, ProductService } from './services';

// Batch functions
async function batchLoadUsers(keys: string[]): Promise<User[]> {
  const users = await UserService.getByIds(keys);
  return keys.map(id => users.find(u => u.id === id) || null);
}

async function batchLoadTeams(keys: string[]): Promise<Team[]> {
  const teams = await TeamService.getByIds(keys);
  return keys.map(id => teams.find(t => t.id === id) || null);
}

async function batchLoadOrdersByUser(userIds: string[]): Promise<Order[][]> {
  const ordersByUser = await OrderService.getByUserIds(userIds);
  return userIds.map(id => ordersByUser[id] || []);
}

async function batchLoadProducts(keys: string[]): Promise<Product[]> {
  const products = await ProductService.getByIds(keys);
  return keys.map(id => products.find(p => p.id === id) || null);
}

export class DataLoader {
  private loaders: {
    user: DataLoader<string, User | null>;
    team: DataLoader<string, Team | null>;
    userOrders: DataLoader<string, Order[]>;
    userTeams: DataLoader<string, Team[]>;
    userPermissions: DataLoader<string, Permission[]>;
    product: DataLoader<string, Product | null>;
    orderCustomer: DataLoader<string, User | null>;
    productCategory: DataLoader<string, Category | null>;
    productBrand: DataLoader<string, Brand | null>;
    orderItems: DataLoader<string, OrderItem[]>;
  };

  constructor() {
    this.loaders = {
      // User loader
      user: new DataLoader(batchLoadUsers, {
        cache: true,
        maxBatchSize: 100,
      }),
      
      // Team loader
      team: new DataLoader(batchLoadTeams, {
        cache: true,
        maxBatchSize: 100,
      }),
      
      // User's orders loader
      userOrders: new DataLoader(
        async (userIds: string[]) => {
          const ordersByUser = await OrderService.getByUserIds(userIds);
          return userIds.map(id => ordersByUser[id] || []);
        },
        { cache: false } // Don't cache as orders change frequently
      ),
      
      // User's teams loader
      userTeams: new DataLoader(
        async (userIds: string[]) => {
          const teamsByUser = await TeamService.getByUserIds(userIds);
          return userIds.map(id => teamsByUser[id] || []);
        },
        { cache: true }
      ),
      
      // User's permissions loader
      userPermissions: new DataLoader(
        async (userIds: string[]) => {
          const permsByUser = await UserService.getPermissionsByIds(userIds);
          return userIds.map(id => permsByUser[id] || []);
        },
        { cache: true }
      ),
      
      // Product loader
      product: new DataLoader(batchLoadProducts, {
        cache: true,
        maxBatchSize: 100,
      }),
      
      // Order's customer loader
      orderCustomer: new DataLoader(
        async (orderIds: string[]) => {
          const customersByOrder = await OrderService.getCustomersByOrderIds(orderIds);
          return orderIds.map(id => customersByOrder[id] || null);
        },
        { cache: true }
      ),
      
      // Product's category loader
      productCategory: new DataLoader(
        async (productIds: string[]) => {
          const categoriesByProduct = await ProductService.getCategoriesByProductIds(productIds);
          return productIds.map(id => categoriesByProduct[id] || null);
        },
        { cache: true }
      ),
      
      // Product's brand loader
      productBrand: new DataLoader(
        async (productIds: string[]) => {
          const brandsByProduct = await ProductService.getBrandsByProductIds(productIds);
          return productIds.map(id => brandsByProduct[id] || null);
        },
        { cache: true }
      ),
      
      // Order's items loader
      orderItems: new DataLoader(
        async (orderIds: string[]) => {
          const itemsByOrder = await OrderService.getItemsByOrderIds(orderIds);
          return orderIds.map(id => itemsByOrder[id] || []);
        },
        { cache: false }
      ),
    };
  }

  // Convenience methods for resolvers
  loadUser(id: string): Promise<User | null> {
    return this.loaders.user.load(id);
  }

  loadTeam(id: string): Promise<Team | null> {
    return this.loaders.team.load(id);
  }

  loadUserOrders(userId: string): Promise<Order[]> {
    return this.loaders.userOrders.load(userId);
  }

  loadUserTeams(userId: string): Promise<Team[]> {
    return this.loaders.userTeams.load(userId);
  }

  loadUserPermissions(userId: string): Promise<Permission[]> {
    return this.loaders.userPermissions.load(userId);
  }

  loadProduct(id: string): Promise<Product | null> {
    return this.loaders.product.load(id);
  }

  loadOrderCustomer(orderId: string): Promise<User | null> {
    return this.loaders.orderCustomer.load(orderId);
  }

  loadProductCategory(productId: string): Promise<Category | null> {
    return this.loaders.productCategory.load(productId);
  }

  loadProductBrand(productId: string): Promise<Brand | null> {
    return this.loaders.productBrand.load(productId);
  }

  loadOrderItems(orderId: string): Promise<OrderItem[]> {
    return this.loaders.orderItems.load(orderId);
  }

  // Clear cache (useful after mutations)
  clearUser(id: string): void {
    this.loaders.user.clear(id);
  }

  clearAll(): void {
    Object.values(this.loaders).forEach(loader => loader.clearAll());
  }
}
```

## 4. Federation Patterns

### 4.1 Federation Schema Design

```graphql
# Federation gateway schema
# extend type statements combine subgraphs

# Users subgraph
extend type Query {
  user(id: ID!): User
  users(filter: UserFilterInput, pagination: PaginationInput): UserConnection!
}

extend type Mutation {
  createUser(input: CreateUserInput!): User!
  updateUser(id: ID!, input: UpdateUserInput!): User!
}

type User @key(fields: "id") {
  id: ID!
  email: String!
  displayName: String!
  role: UserRole!
  status: UserStatus!
  avatarUrl: URL
  createdAt: DateTime!
  preferences: UserPreferences!
  
  # Product associations (from Products subgraph)
  wishlist: [Product!]!
  recentlyViewed: [Product!]!
  orders: [Order!]!
}

enum UserRole {
  USER
  ADMIN
  SUPER_ADMIN
  SERVICE_ACCOUNT
  READ_ONLY
}

enum UserStatus {
  ACTIVE
  INACTIVE
  SUSPENDED
  DELETED
}

# Products subgraph
extend type Query {
  product(id: ID, slug: String): Product
  products(filter: ProductFilterInput, pagination: PaginationInput): ProductConnection!
  searchProducts(query: String!): [SearchResult!]!
}

extend type Mutation {
  createProduct(input: CreateProductInput!): Product!
  updateProduct(id: ID!, input: UpdateProductInput!): Product!
}

type Product @key(fields: "id") @key(fields: "sku") {
  id: ID!
  sku: String!
  name: String!
  slug: String!
  description: String!
  price: Money!
  images: [ProductImage!]!
  inventory: InventoryStatus!
  category: Category!
  
  # Reviews (from Reviews subgraph)
  reviews: [Review!]!
  averageRating: Float
  
  # Owner reference (from Users subgraph)
  createdBy: User!
}

type Category @key(fields: "id") {
  id: ID!
  name: String!
  slug: String!
  products(first: Int): [Product!]!
  parent: Category
  children: [Category!]!
}

# Orders subgraph
extend type Query {
  order(id: ID!): Order
  orders(filter: OrderFilterInput, pagination: PaginationInput): OrderConnection!
}

extend type Mutation {
  createOrder(input: CreateOrderInput!): Order!
  cancelOrder(id: ID!): Order!
}

type Order @key(fields: "id") {
  id: ID!
  orderNumber: String!
  status: OrderStatus!
  total: Money!
  
  # Customer reference (from Users subgraph)
  customer: User!
  
  # Products reference (from Products subgraph)
  items: [OrderItem!]!
}

# Reviews subgraph
extend type Query {
  reviews(productId: ID!): [Review!]!
}

type Review @key(fields: "id") {
  id: ID!
  rating: Int!
  content: String!
  
  # References
  product: Product!
  author: User!
}
```

### 4.2 Subgraph Implementation

```typescript
// products subgraph - Apollo Server

import { ApolloServer } from '@apollo/server';
import { startStandaloneServer } from '@apollo/server/standalone';
import { buildSubgraphSchema } from '@apollo/subgraph';
import { createDirectives } from './directives';
import { ProductService } from './services/product.service';
import { resolvers } from './resolvers';

const PRODUCT_SERVICE = new ProductService();

const typeDefs = `
  type Product @key(fields: "id") @key(fields: "sku") {
    id: ID!
    sku: String!
    name: String!
    slug: String!
    description: String!
    price: Money!
    compareAtPrice: Money
    category: Category!
    brand: Brand
    images: [ProductImage!]!
    inventory: InventoryStatus!
    status: ProductStatus!
    createdAt: DateTime!
    updatedAt: DateTime!
    
    # Entity reference for federation
    categoryId: ID!
    brandId: ID
    createdById: ID!
    
    # Extension fields (resolved by other subgraphs)
    reviews: [Review!]!
    createdBy: User!
  }
  
  extend type Query {
    product(id: ID, slug: String): Product
    products(filter: ProductFilterInput, pagination: PaginationInput): ProductConnection!
    searchProducts(query: String!): [SearchResult!]!
  }
  
  extend type Mutation {
    createProduct(input: CreateProductInput!): Product!
    updateProduct(id: ID!, input: UpdateProductInput!): Product!
  }
`;

const schema = buildSubgraphSchema({ typeDefs, resolvers });

const server = new ApolloServer({
  schema,
  plugins: [
    // Federation tracing plugin
    import('@apollo/server-plugin-landing-pages-graphql-federation'),
  ],
});

const { url } = await startStandaloneServer(server, {
  context: async ({ req }) => ({
    authorization: req.headers.authorization,
  }),
  listen: { port: 4001 },
});

console.log(`Products subgraph ready at ${url}`);
```

```typescript
// users subgraph

const typeDefs = `
  type User @key(fields: "id") {
    id: ID!
    email: String!
    displayName: String!
    firstName: String
    lastName: String
    avatarUrl: URL
    role: UserRole!
    status: UserStatus!
    preferences: UserPreferences!
    createdAt: DateTime!
    updatedAt: DateTime!
    
    # Entity references for other subgraphs
    wishlist: [Product!]!
    orders: [Order!]!
    createdProducts: [Product!]!
  }
  
  extend type Query {
    me: User
    user(id: ID!): User
    users(filter: UserFilterInput): UserConnection!
  }
  
  extend type Mutation {
    createUser(input: CreateUserInput!): User!
    updateUser(id: ID!, input: UpdateUserInput!): User!
  }
`;
```

## 5. Subscription Patterns

### 5.1 Subscription Resolver Implementation

```typescript
// subscriptions/resolvers.ts

import { PubSub } from 'graphql-subscriptions';

const pubsub = new PubSub();

// Event names
const EVENTS = {
  ORDER_CREATED: 'ORDER_CREATED',
  ORDER_UPDATED: 'ORDER_UPDATED',
  ORDER_STATUS_CHANGED: 'ORDER_STATUS_CHANGED',
  PRODUCT_UPDATED: 'PRODUCT_UPDATED',
  PRODUCT_INVENTORY_CHANGED: 'PRODUCT_INVENTORY_CHANGED',
  CART_UPDATED: 'CART_UPDATED',
  NOTIFICATION: 'NOTIFICATION',
  MESSAGE_RECEIVED: 'MESSAGE_RECEIVED',
};

const subscriptionResolvers = {
  Subscription: {
    // Order subscriptions
    orderStatusChanged: {
      subscribe: async function* (
        _: unknown,
        { orderId }: { orderId: string },
        context: Context
      ) {
        // Verify subscription authorization
        await OrderService.verifyAccess(orderId, context.user?.id);
        
        const order = await OrderService.get(orderId);
        const lastStatus = order.status;
        
        for await (const event of OrderService.subscribeToStatusChanges(orderId)) {
          if (event.status !== lastStatus) {
            lastStatus = event.status;
            yield {
              orderStatusChanged: {
                orderId,
                previousStatus: event.previousStatus,
                newStatus: event.newStatus,
                timestamp: event.timestamp,
                order: await OrderService.get(orderId),
              },
            };
          }
        }
      },
    },

    myOrdersUpdated: {
      subscribe: async function* (
        _: unknown,
        __: unknown,
        context: Context
      ) {
        if (!context.user) {
          throw new AuthError('Not authenticated');
        }
        
        for await (const event of OrderService.subscribeToCustomerOrders(context.user.id)) {
          yield { myOrdersUpdated: event };
        }
      },
    },

    // Product subscriptions
    productUpdated: {
      subscribe: async (
        _: unknown,
        { productId }: { productId: string }
      ) {
        return pubsub.asyncIterator([`${EVENTS.PRODUCT_UPDATED}.${productId}`]);
      },
    },

    productInventoryChanged: {
      subscribe: async (
        _: unknown,
        { productIds }: { productIds: string[] }
      ) {
        const topics = productIds.map(id => `${EVENTS.PRODUCT_INVENTORY_CHANGED}.${id}`);
        return pubsub.asyncIterator(topics);
      },
    },

    // Cart subscriptions
    cartUpdated: {
      subscribe: async (
        _: unknown,
        __: unknown,
        context: Context
      ) {
        if (!context.user) {
          // Use session ID for anonymous users
          const sessionId = context.sessionId;
          if (!sessionId) {
            throw new AuthError('Not authenticated or no session');
          }
          return pubsub.asyncIterator([`${EVENTS.CART_UPDATED}.session.${sessionId}`]);
        }
        return pubsub.asyncIterator([`${EVENTS.CART_UPDATED}.user.${context.user.id}`]);
      },
    },

    // Notification subscriptions
    notificationReceived: {
      subscribe: async (
        _: unknown,
        __: unknown,
        context: Context
      ) {
        if (!context.user) {
          throw new AuthError('Not authenticated');
        }
        return pubsub.asyncIterator([`${EVENTS.NOTIFICATION}.${context.user.id}`]);
      },
    },

    // Chat subscriptions
    messageReceived: {
      subscribe: async (
        _: unknown,
        { threadId }: { threadId: string },
        context: Context
      ) {
        // Verify thread access
        await MessageService.verifyThreadAccess(threadId, context.user?.id);
        return pubsub.asyncIterator([`${EVENTS.MESSAGE_RECEIVED}.${threadId}`]);
      },
    },
  },

  // Publish helpers (called from mutations)
  Order: {
    publishStatusChange: async (order: Order, previousStatus: OrderStatus) => {
      await pubsub.publish(`${EVENTS.ORDER_STATUS_CHANGED}.${order.id}`, {
        orderStatusChanged: {
          orderId: order.id,
          previousStatus,
          newStatus: order.status,
          timestamp: new Date(),
          order,
        },
      });
    },
  },

  Product: {
    publishInventoryChange: async (productId: string, oldQty: number, newQty: number) => {
      await pubsub.publish(`${EVENTS.PRODUCT_INVENTORY_CHANGED}.${productId}`, {
        productInventoryChanged: {
          productId,
          previousQuantity: oldQty,
          newQuantity: newQty,
          timestamp: new Date(),
        },
      });
    },
  },
};
```

## 6. Decision Matrices

### 6.1 Schema Design Decision Matrix

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                          GraphQL Schema Design Decision Matrix                           │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Decision                    │ Choose This When                  │ Choose That When        │
├─────────────────────────────┼───────────────────────────────────┼────────────────────────┤
│ Connection vs List          │ Need pagination                   │ Fixed, small lists      │
│                             │ Need totalCount                   │ Don't need totalCount   │
│                             │ Need cursor-based navigation       │ Simple offset pagination│
├─────────────────────────────┼───────────────────────────────────┼────────────────────────┤
│ Embedded vs Reference       │ Always belongs to parent          │ Shared across entities  │
│                             │ Never queried standalone          │ Queried independently   │
│                             │ No update cascade needed           │ Updates should cascade │
├─────────────────────────────┼───────────────────────────────────┼────────────────────────┤
│ Input vs Inline            │ Reuse across mutations             │ Unique to one mutation  │
│                             │ Complex validation logic          │ Simple transformation   │
├─────────────────────────────┼───────────────────────────────────┼────────────────────────┤
│ Single vs Multiple Types   │ Clear entity distinction          │ Overlapping concerns   │
│ for Similar Data            │ Different update patterns          │ Shared fields dominate │
│                             │ Performance concerns               │ Easier querying        │
├─────────────────────────────┼───────────────────────────────────┼────────────────────────┤
│ Interface vs Union          │ Shared fields exist               │ No shared fields       │
│                             │ Can return in same query          │ Mutually exclusive     │
│                             │ Common handling logic             │ Different result shapes │
├─────────────────────────────┼───────────────────────────────────┼────────────────────────┤
│ Custom Scalar vs String     │ Strong typing needed              │ Quick prototyping      │
│                             │ Validation at schema level        │ Schema flexibility     │
│                             │ Self-documenting                  │ Minimal boilerplate    │
├─────────────────────────────┼───────────────────────────────────┼────────────────────────┤
│ Nullable vs Non-null        │ Field can be absent               │ Always present required │
│                             │ DB NULL semantic matches          │ Business logic requires │
│                             │ Partial objects                   │ Breaking change if null │
└─────────────────────────────┴───────────────────────────────────┴────────────────────────┘
```

### 6.2 Query Optimization Decision Matrix

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                            Query Optimization Decision Matrix                            │
├─────────────────────────────────────────────────────────────────────────────┬───────────┤
│ Scenario                                                            │ Solution         │
├─────────────────────────────────────────────────────────────────────┼─────────────────┤
│ Fetching 100+ related objects causing N+1                           │ Use DataLoader   │
│                                                                     │ batch loading    │
├─────────────────────────────────────────────────────────────────────┼─────────────────┤
│ Deep nested queries with same subfields                            │ Use fragments    │
│                                                                     │ with spread      │
├─────────────────────────────────────────────────────────────────────┼─────────────────┤
│ Expensive computation repeated for same data                       │ Use field        │
│                                                                     │-level caching   │
├─────────────────────────────────────────────────────────────────────┼─────────────────┤
│ Large list queries where client paginates                          │ Use connections  │
│                                                                     │ with cursor-based│
├─────────────────────────────────────────────────────────────────────┼─────────────────┤
│ Client only needs specific fields, not full object                 │ Use relay-style  │
│                                                                     │ field selections │
├─────────────────────────────────────────────────────────────────────┼─────────────────┤
│ Expensive validation that doesn't affect response                  │ Use @defer       │
│                                                                     │ for non-critical │
│                                                                     │ validation errors │
├─────────────────────────────────────────────────────────────────────┼─────────────────┤
│ Queries that should always return fresh data                       │ Bypass cache     │
│                                                                     │ with no-cache    │
│                                                                     │ directive        │
├─────────────────────────────────────────────────────────────────────┼─────────────────┤
│ Complex queries with multiple optional filters                     │ Use query        │
│                                                                     │ complexity       │
│                                                                     │ analysis         │
└─────────────────────────────────────────────────────────────────────┴─────────────────┘
```

## 7. Anti-Patterns

### 7.1 Common GraphQL Anti-Patterns

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                            GraphQL Anti-Patterns to Avoid                                │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Anti-Pattern                    │ Problem                       │ Solution                │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ N+1 queries                    │ Performance degradation        │ Use DataLoader          │
│                                 │ Too many DB round trips        │ batch loading           │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Deep nesting without limit     │ Memory exhaustion              │ Use query depth limit  │
│                                 │ Exponential query complexity   │ and complexity limits  │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Schema without pagination      │ Memory issues with large sets  │ Use Connection pattern │
│                                 │ No cursor-based navigation     │ with first/after        │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Type name collisions           │ Federation issues              │ Use namespacing         │
│                                 │ Unclear ownership              │ (User_V1, Product_V2)   │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Using REST patterns in GraphQL │ Missing GraphQL benefits       │ Use GraphQL-native     │
│                                 │ Overfetching/underfetching    │ patterns               │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No error handling strategy     │ Unclear error responses       │ Use error types        │
│                                 │ Client confusion               │ with extensions        │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Mutations returning too much   │ Unnecessary data transfer      │ Use @include/@skip     │
│                                 │ Security concerns              │ or separate queries    │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Overly generic types           │ Loss of type safety           │ Use specific types      │
│ (JSON, Any, etc.)              │ No validation                  │ with validation        │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Missing field deprecation      │ API evolution difficulties    │ Use @deprecated        │
│                                 │ Client confusion               │ with reason            │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No caching strategy            │ Repeated expensive queries    │ Implement Persisted    │
│                                 │ Client-side caching issues     │ Queries + CDN cache    │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Ignoring query complexity      │ DoS vulnerabilities           │ Set complexity limits  │
│                                 │ Server overload               │ and depth limits       │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Missing validation             │ Schema accepts anything       │ Use input validation   │
│                                 │ Hard to debug                  │ with custom scalars   │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Improper null handling         │ Unexpected errors              │ Use NonNull carefully  │
│                                 │ Partial data returns          │ Plan for nullability  │
└─────────────────────────────────┴───────────────────────────────┴────────────────────────┘
```

### 7.2 Bad vs Good Examples

```graphql
# BAD: Deep nesting without limits
query DeepNesting {
  orders {
    customer {
      orders {  # Can keep going...
        customer {
          orders {
            # Infinite! Memory exhaustion
          }
        }
      }
    }
  }
}

# GOOD: Depth limit with pagination
query OrdersWithLimits {
  orders(first: 10) {
    edges {
      node {
        customer {
          id
          displayName
          recentOrders: orders(first: 3) {  # Limited depth
            edges {
              node {
                orderNumber
                total
              }
            }
          }
        }
      }
    }
  }
}

# BAD: N+1 in nested query
query BadQuery {
  users(first: 100) {
    id
    orders {  # Each order triggers separate DB query
      id
      items {  # Each item triggers another query
        product {  # Another query per product
          id
          name
        }
      }
    }
  }
}

# GOOD: Use DataLoader for batch loading
query GoodQuery {
  users(first: 100) {
    edges {
      node {
        id
        orders(first: 10) {  # DataLoader batches these
          edges {
            node {
              id
              items(first: 20) {  # Batched together
                edges {
                  node {
                    product {
                      id  # All products batched in one query
                      name
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
}

# BAD: Overly generic type
type Query {
  search(type: String!, id: String!): JSON  # No type safety!
}

# GOOD: Specific union type
type Query {
  search(query: String!): SearchResultUnion!
}

union SearchResultUnion = Product | Category | Brand | Page

# BAD: No pagination
type Query {
  allProducts: [Product!]!  # Could be millions!
}

# GOOD: Cursor-based pagination
type Query {
  products(after: String, first: Int, before: String, last: Int): ProductConnection!
}
```

## 8. Performance Best Practices

### 8.1 Query Performance Guidelines

```markdown
1. Field Resolution Optimization
   - Use DataLoader for all relationship fields
   - Batch database queries by parent IDs
   - Cache computed fields appropriately
   - Avoid N+1 queries at all costs

2. Pagination Best Practices
   - Always use cursor-based pagination for large datasets
   - Set reasonable default limits (10-50 items)
   - Enforce maximum limits (never allow unlimited)
   - Use count queries sparingly (expensive)

3. Query Complexity
   - Set maximum query depth (recommend: 10-15)
   - Set maximum query complexity
   - Use complexity multipliers for expensive fields
   - Monitor and alert on high complexity queries

4. Response Caching
   - Implement Persisted Queries
   - Use CDN caching for public queries
   - Implement field-level cache directives
   - Consider @defer for non-critical fields

5. Request Validation
   - Validate all input types
   - Use custom scalars for strict validation
   - Reject overly large queries early
   - Check resource limits before execution
```

### 8.2 Security Best Practices

```markdown
1. Authentication & Authorization
   - Always authenticate queries and mutations
   - Implement field-level authorization
   - Use directive-based auth for reusable rules
   - Never expose sensitive fields without auth

2. Rate Limiting
   - Implement per-user rate limits
   - Consider query complexity in limits
   - Use token bucket algorithm
   - Return appropriate errors on limit exceeded

3. Query Validation
   - Set maximum depth
   - Set maximum complexity
   - Set maximum aliases
   - Set maximum directive depth

4. Error Handling
   - Don't expose internal errors
   - Use error codes for client handling
   - Log errors server-side
   - Sanitize error messages

5. Sensitive Data
   - Never include passwords in responses
   - Mask sensitive fields (SSN, credit cards)
   - Use separate endpoints for admin data
   - Implement field-level permissions
```

---

## Links

### Official Documentation
- [GraphQL Specification](https://spec.graphql.org/)
- [GraphQL Foundation](https://graphql.org/foundation)
- [Apollo GraphQL](https://www.apollographql.com/)
- [Apollo Federation](https://www.apollographql.com/docs/federation/)
- [Apollo Server](https://www.apollographql.com/docs/apollo-server/)

### Schema Design
- [Schema Design Best Practices](https://www.apollographql.com/docs/apollo-server/schema/schema/)
- [Schema Stitching](https://www.apollographql.com/docs/apollo-server/schema/schema-stitching/)
- [GraphQL Schema Language](https://graphql.org/learn/schema/)

### Data Loading
- [DataLoader Documentation](https://github.com/graphql/dataloader)
- [Avoiding N+1 Queries](https://www.apollographql.com/docs/apollo-server/data/data-loader/)
- [Batching and Caching](https://graphql.org/graphql-js/object-nature/)

### Federation
- [Apollo Federation Docs](https://www.apollographql.com/docs/federation/)
- [Federation Spec](https://www.apollographql.com/docs/federation/federation-spec/)
- [Subgraph Implementation](https://www.apollographql.com/docs/federation/subgraphs/)

### Subscriptions
- [GraphQL Subscriptions](https://www.apollographql.com/docs/react/data/subscriptions/)
- [PubSub Implementation](https://github.com/apollographql/graphql-subscriptions)
- [WebSocket Protocol](https://github.com/enisdenjo/graphql-ws)

### Performance
- [Query Performance](https://www.apollographql.com/docs/apollo-server/performance/)
- [Caching](https://www.apollographql.com/docs/apollo-server/performance/caching/)
- [Persisted Queries](https://www.apollographql.com/docs/apollo-server/performance/api-key/)

### Security
- [GraphQL Security](https://www.apollographql.com/docs/apollo-server/security/)
- [Query Complexity](https://www.apollographql.com/docs/apollo-server/validation/checks/)
- [Rate Limiting](https://www.apollographql.com/docs/apollo-server/security/rate-limits/)

### Tools
- [GraphiQL](https://github.com/graphql/graphiql)
- [Apollo Studio](https://studio.apollographql.com/)
- [Prisma](https://www.prisma.io/)
- [GraphQL Code Generator](https://www.graphql-code-generator.com/)
- [eslint-plugin-graphql](https://github.com/B2noor/eslint-plugin-graphql)

### Testing
- [Apollo Testing](https://www.apollographql.com/docs/apollo-server/testing/)
- [Jest + GraphQL](https://www.apollographql.com/docs/react/testing/)
- [Mocking](https://www.apollographql.com/docs/react/api/react-testing/)

### Learning
- [How to GraphQL](https://www.howtographql.com/)
- [GraphQL Learning](https://graphql.org/learn/)
- [Apollo Odyssey](https://www.apollographql.com/docs/odyssey/)