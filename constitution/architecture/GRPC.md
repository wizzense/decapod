# GRPC.md - gRPC Architecture Standards

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

---

## 1. Protocol Buffer Fundamentals

### 1.1 Protobuf Version and Syntax

```protobuf
// proto3 syntax - REQUIRED for all new services
syntax = "proto3";

package myservice.v1;

option go_package = "github.com/example/myservice/v1;v1";
option java_package = "com.example.myservice.v1";
option java_multiple_files = true;
option java_outer_classname = "MyServiceProto";
```

### 1.2 Scalar Types Mapping

```protobuf
// Protocol Buffer to Language Type Mappings

message TypeMappings {
    // proto Type        // Go Type           // Java Type           // Python Type
    string              // string             // String               // str
    int32               // int32              // int                  // int
    int64               // int64              // long                 // int
    uint32              // uint32             // int                  // int
    uint64              // uint64             // long                 // int
    float               // float32            // float                // float
    double              // float64            // double               // float
    bool                // bool               // boolean              // bool
    bytes               // []byte             // ByteString           // bytes
    
    // Well-known types
    google.protobuf.Timestamp   timestamp = 1;  // time.Time           // Instant
    google.protobuf.Duration    duration = 2;   // time.Duration       // Duration
    google.protobuf.Empty       empty = 3;      // struct{}            // None
    google.protobuf.Struct      struct = 4;     // map[string,any]     // dict
    google.protobuf.Value       value = 5;      // interface{}         // Any
    google.protobuf.ListValue   list = 6;       // []interface{}       // list
    google.protobuf.BoolValue   bool = 7;        // *bool               // Optional[bool]
    google.protobuf.StringValue str = 8;        // *string             // Optional[str]
    google.protobuf.Int32Value  num = 9;        // *int32              // Optional[int]
}
```

### 1.3 Field Rules and Cardinalities

```protobuf
// Field rules determine cardinality and optionality

message FieldRulesExample {
    // Single values (singular) - default for proto3
    string name = 1;              // Optional singular scalar
    User user = 2;                // Optional singular message
    
    // Repeated fields - zero or more
    repeated string aliases = 3;  // Repeated scalar
    repeated User friends = 4;     // Repeated message
    
    // Map fields - key-value collections
    map<string, int32> scores = 5;
    map<string, User> users_by_name = 6;
    map<int64, string> id_to_email = 7;
    
    // OneOf - mutually exclusive fields
    oneof content {
        TextContent text = 8;
        ImageContent image = 9;
        AudioContent audio = 10;
    }
    
    // Reserved fields - prevent field number reuse
    reserved 100 to 105;
    reserved "deprecated_field", "old_name";
}

// Maps have specific constraints
message MapConstraints {
    // Keys: any scalar type except floating point or bytes
    // Values: any type except another map
    
    map<string, string> string_to_string = 1;   // Valid
    map<int32, User> int_to_user = 2;           // Valid
    map<string, map<string, int>> nested = 3;  // INVALID - maps cannot be map values
    
    // Alternative for nested maps
    map<string, NestedEntry> nested_proper = 4;  // Valid
    
    message NestedEntry {
        map<string, int> inner = 1;
    }
}

// OneOf behavior
message OneOfExample {
    oneof result {
        SuccessResponse success = 1;
        ErrorResponse error = 2;
        LoadingState loading = 3;
    }
    // Setting 'success' clears 'error' and 'loading'
    // Setting 'error' clears 'success' and 'loading'
}
```

## 2. Service Definition Patterns

### 2.1 Basic Service Structure

```protobuf
// Complete user service definition
syntax = "proto3";

package user.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/empty.proto";
import "google/protobuf/wrappers.proto";
import "validate/validate.proto";

option go_package = "github.com/example/user/v1;userv1";
option java_package = "com.example.user.v1";
option java_multiple_files = true;

// UserService handles user management operations
service UserService {
    // Unary RPC - single request, single response
    rpc GetUser(GetUserRequest) returns (GetUserResponse);
    
    // Server streaming - single request, multiple responses
    rpc ListUserEvents(ListUserEventsRequest) returns (stream UserEvent);
    
    // Client streaming - multiple requests, single response
    rpc StreamUserMetrics(stream UserMetric) returns (AggregateMetricsResponse);
    
    // Bidirectional streaming - multiple requests, multiple responses
    rpc StreamChatMessages(stream ChatMessage) returns (stream ChatMessage);
    
    // Batch operations
    rpc BatchGetUsers(BatchGetUsersRequest) returns (BatchGetUsersResponse);
    
    // Health check (conventional)
    rpc HealthCheck(google.protobuf.Empty) returns (HealthCheckResponse);
}

// Message definitions for UserService
message User {
    string id = 1 [(validate.rules).string.uuid = true];
    string email = 2 [(validate.rules).string.email = true];
    string display_name = 3 [(validate.rules).string.min_len = 1];
    UserRole role = 4;
    google.protobuf.Timestamp created_at = 5;
    google.protobuf.Timestamp updated_at = 6;
    google.protobuf.Timestamp last_login_at = 7;
    UserMetadata metadata = 8;
    bool email_verified = 9;
    bool account_locked = 10;
}

enum UserRole {
    USER_ROLE_UNSPECIFIED = 0;
    USER_ROLE_USER = 1;
    USER_ROLE_ADMIN = 2;
    USER_ROLE_SUPER_ADMIN = 3;
    USER_ROLE_SERVICE_ACCOUNT = 4;
    USER_ROLE_READ_ONLY = 5;
}

message UserMetadata {
    map<string, string> custom_attributes = 1;
    repeated string enrolled_features = 2;
    string subscription_tier = 3;
    repeated string allowed_origins = 4;
}

message GetUserRequest {
    string user_id = 1 [(validate.rules).string.uuid = true];
    repeated string fields = 2;  // Partial response support
}

message GetUserResponse {
    User user = 1;
    string request_id = 2;
}

message ListUserEventsRequest {
    string user_id = 1 [(validate.rules).string.uuid = true];
    EventType event_type = 2;
    google.protobuf.Timestamp start_time = 3;
    google.protobuf.Timestamp end_time = 4;
    int32 page_size = 5 [(validate.rules).int32 = {gte: 1, lte: 1000}];
    string page_token = 6;
}

enum EventType {
    EVENT_TYPE_UNSPECIFIED = 0;
    EVENT_TYPE_LOGIN = 1;
    EVENT_TYPE_LOGOUT = 2;
    EVENT_TYPE_PASSWORD_CHANGE = 3;
    EVENT_TYPE_EMAIL_CHANGE = 4;
    EVENT_TYPE_PROFILE_UPDATE = 5;
    EVENT_TYPE_ACCOUNT_LOCK = 6;
    EVENT_TYPE_ACCOUNT_UNLOCK = 7;
    EVENT_TYPE_PERMISSION_CHANGE = 8;
}

message UserEvent {
    string event_id = 1;
    string user_id = 2;
    EventType event_type = 3;
    google.protobuf.Timestamp occurred_at = 4;
    map<string, string> event_data = 5;
    string ip_address = 6;
    string user_agent = 7;
}

message ListUserEventsResponse {
    repeated UserEvent events = 1;
    string next_page_token = 2;
    int32 total_count = 3;
}
```

### 2.2 Complete E-Commerce Service Example

```protobuf
syntax = "proto3";

package ecommerce.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/duration.proto";
import "google/protobuf/empty.proto";
import "google/protobuf/wrappers.proto";
import "validate/validate.proto";

option go_package = "github.com/example/ecommerce/v1;ecommercev1";
option java_package = "com.example.ecommerce.v1";
option java_multiple_files = true;

// ProductCatalogService manages product catalog
service ProductCatalogService {
    rpc GetProduct(GetProductRequest) returns (Product);
    rpc ListProducts(ListProductsRequest) returns (ListProductsResponse);
    rpc SearchProducts(SearchProductsRequest) returns (SearchProductsResponse);
    rpc CreateProduct(CreateProductRequest) returns (Product);
    rpc UpdateProduct(UpdateProductRequest) returns (Product);
    rpc DeleteProduct(DeleteProductRequest) returns (google.protobuf.Empty);
    rpc StreamProductUpdates(StreamProductUpdatesRequest) returns (stream ProductUpdate);
    rpc BatchGetProducts(BatchGetProductsRequest) returns (BatchGetProductsResponse);
}

// OrderService handles order processing
service OrderService {
    rpc CreateOrder(CreateOrderRequest) returns (Order);
    rpc GetOrder(GetOrderRequest) returns (Order);
    rpc ListOrders(ListOrdersRequest) returns (ListOrdersResponse);
    rpc CancelOrder(CancelOrderRequest) returns (Order);
    rpc StreamOrderUpdates(StreamOrderUpdatesRequest) returns (stream OrderUpdate);
    rpc UpdateOrderStatus(UpdateOrderStatusRequest) returns (Order);
}

// InventoryService manages inventory
service InventoryService {
    rpc CheckAvailability(CheckAvailabilityRequest) returns (AvailabilityResponse);
    rpc ReserveInventory(ReserveInventoryRequest) returns (Reservation);
    rpc ReleaseInventory(ReleaseInventoryRequest) returns (google.protobuf.Empty);
    rpc AdjustInventory(AdjustInventoryRequest) returns (InventoryAdjustment);
    rpc StreamInventoryUpdates(StreamInventoryUpdatesRequest) returns (stream InventoryUpdate);
}

// PaymentService handles payments
service PaymentService {
    rpc ProcessPayment(ProcessPaymentRequest) returns (PaymentResult);
    rpc RefundPayment(RefundPaymentRequest) returns (RefundResult);
    rpc GetPayment(GetPaymentRequest) returns (Payment);
    rpc ListPayments(ListPaymentsRequest) returns (ListPaymentsResponse);
    rpc StreamPaymentUpdates(StreamPaymentUpdatesRequest) returns (stream PaymentUpdate);
}

// CartService handles shopping cart
service CartService {
    rpc GetCart(GetCartRequest) returns (Cart);
    rpc AddItem(AddItemRequest) returns (Cart);
    rpc UpdateItemQuantity(UpdateItemQuantityRequest) returns (Cart);
    rpc RemoveItem(RemoveItemRequest) returns (Cart);
    rpc ClearCart(ClearCartRequest) returns (google.protobuf.Empty);
    rpc StreamCartUpdates(StreamCartUpdatesRequest) returns (stream CartUpdate);
}

// Product Messages
message Product {
    string id = 1;
    string sku = 2 [(validate.rules).string.pattern = "^[A-Z]{3}-[0-9]{6}$"];
    string name = 3;
    string description = 4;
    ProductCategory category = 5;
    repeated ProductVariant variants = 6;
    Money price = 7;
    ProductInventory inventory = 8;
    ProductImages images = 9;
    ProductAttributes attributes = 10;
    ProductStatus status = 11;
    google.protobuf.Timestamp created_at = 12;
    google.protobuf.Timestamp updated_at = 13;
    bool active = 14;
    repeated string tags = 15;
}

enum ProductCategory {
    PRODUCT_CATEGORY_UNSPECIFIED = 0;
    PRODUCT_CATEGORY_ELECTRONICS = 1;
    PRODUCT_CATEGORY_CLOTHING = 2;
    PRODUCT_CATEGORY_HOME_AND_GARDEN = 3;
    PRODUCT_CATEGORY_SPORTS = 4;
    PRODUCT_CATEGORY_BOOKS = 5;
    PRODUCT_CATEGORY_TOYS = 6;
    PRODUCT_CATEGORY_FOOD = 7;
    PRODUCT_CATEGORY_BEAUTY = 8;
    PRODUCT_CATEGORY_AUTO = 9;
    PRODUCT_CATEGORY_INDUSTRIAL = 10;
}

message ProductVariant {
    string id = 1;
    string name = 2;
    map<string, string> attributes = 3;  // size, color, etc.
    string sku = 4;
    Money price_modifier = 5;
    int32 inventory_count = 6;
}

message ProductInventory {
    int32 total_quantity = 1;
    int32 available_quantity = 2;
    int32 reserved_quantity = 3;
    int32 reorder_threshold = 4;
    bool low_stock_alert = 5;
    string warehouse_location = 6;
}

message ProductImages {
    repeated ProductImage images = 1;
    string primary_image_url = 2;
}

message ProductImage {
    string url = 1;
    string alt_text = 2;
    int32 width = 3;
    int32 height = 4;
    int32 sort_order = 5;
    bool is_primary = 6;
}

message ProductAttributes {
    map<string, string> attributes = 1;
    map<string, repeated string> multi_valued_attributes = 2;
    ProductSpecifications specifications = 3;
}

message ProductSpecifications {
    double weight = 1;
    string weight_unit = 2;
    Dimensions dimensions = 3;
    repeated string materials = 4;
    string origin_country = 5;
}

message Dimensions {
    double length = 1;
    double width = 2;
    double height = 3;
    string unit = 4;
}

enum ProductStatus {
    PRODUCT_STATUS_UNSPECIFIED = 0;
    PRODUCT_STATUS_DRAFT = 1;
    PRODUCT_STATUS_ACTIVE = 2;
    PRODUCT_STATUS_INACTIVE = 3;
    PRODUCT_STATUS_DISCONTINUED = 4;
    PRODUCT_STATUS_PENDING_REVIEW = 5;
}

// Money type for all currency values
message Money {
    string currency_code = 1 [(validate.rules).string.len = 3];
    int64 amount = 2;  // Amount in smallest currency unit (cents)
    int32 decimal_places = 3;
}

// Product Request/Response Messages
message GetProductRequest {
    string product_id = 1;
    repeated string fields = 2;
}

message ListProductsRequest {
    ProductCategory category = 1;
    ProductStatus status = 2;
    int32 page_size = 3 [(validate.rules).int32 = {gte: 1, lte: 100}];
    string page_token = 4;
    string order_by = 5;
    bool ascending = 6;
}

message ListProductsResponse {
    repeated Product products = 1;
    string next_page_token = 2;
    int32 total_count = 3;
}

message SearchProductsRequest {
    string query = 1;
    repeated ProductCategory categories = 2;
    PriceRange price_range = 3;
    repeated string tags = 4;
    double min_rating = 5;
    int32 page_size = 6 [(validate.rules).int32 = {gte: 1, lte: 100}];
    string page_token = 7;
}

message PriceRange {
    Money min_price = 1;
    Money max_price = 2;
}

message SearchProductsResponse {
    repeated SearchResult results = 1;
    FacetData facets = 2;
    string next_page_token = 3;
    int32 total_count = 4;
}

message SearchResult {
    Product product = 1;
    double relevance_score = 2;
    repeated string matched_terms = 3;
}

message FacetData {
    repeated CategoryFacet category_facets = 1;
    repeated PriceFacet price_facets = 2;
    repeated RatingFacet rating_facets = 3;
}

message CategoryFacet {
    ProductCategory category = 1;
    int32 count = 2;
}

message PriceFacet {
    string label = 1;
    Money min_price = 2;
    Money max_price = 3;
    int32 count = 4;
}

message RatingFacet {
    double min_rating = 1;
    int32 count = 2;
}

message CreateProductRequest {
    Product product = 1 [(validate.rules).message.required = true];
}

message UpdateProductRequest {
    string product_id = 1;
    Product product = 2 [(validate.rules).message.required = true];
    google.protobuf.FieldMask update_mask = 3;
}

message DeleteProductRequest {
    string product_id = 1;
    bool force = 2;
}

message StreamProductUpdatesRequest {
    repeated string product_ids = 1;
    bool include_inventory_updates = 2;
    bool include_price_updates = 3;
}

message ProductUpdate {
    string product_id = 1;
    UpdateType update_type = 2;
    Product product = 3;
    InventoryUpdate inventory_update = 4;
    google.protobuf.Timestamp timestamp = 5;
}

enum UpdateType {
    UPDATE_TYPE_UNSPECIFIED = 0;
    UPDATE_TYPE_CREATED = 1;
    UPDATE_TYPE_UPDATED = 2;
    UPDATE_TYPE_DELETED = 3;
    UPDATE_TYPE_INVENTORY_CHANGED = 4;
    UPDATE_TYPE_PRICE_CHANGED = 5;
}

message InventoryUpdate {
    int32 previous_quantity = 1;
    int32 new_quantity = 2;
    string reason = 3;
    string warehouse_id = 4;
}

message BatchGetProductsRequest {
    repeated string product_ids = 1;
    repeated string fields = 2;
}

message BatchGetProductsResponse {
    repeated Product products = 1;
    repeated NotFoundResult not_found = 2;
}

message NotFoundResult {
    string id = 1;
    string error_message = 2;
}

// Order Messages
message Order {
    string id = 1;
    string customer_id = 2;
    OrderStatus status = 3;
    repeated OrderItem items = 4;
    Money subtotal = 5;
    Money tax = 6;
    Money shipping_cost = 7;
    Money discount = 8;
    Money total = 9;
    ShippingAddress shipping_address = 10;
    BillingAddress billing_address = 11;
    PaymentInfo payment_info = 12;
    string tracking_number = 13;
    google.protobuf.Timestamp created_at = 14;
    google.protobuf.Timestamp updated_at = 15;
    google.protobuf.Timestamp shipped_at = 16;
    google.protobuf.Timestamp delivered_at = 17;
    repeated OrderEvent history = 18;
}

enum OrderStatus {
    ORDER_STATUS_UNSPECIFIED = 0;
    ORDER_STATUS_PENDING = 1;
    ORDER_STATUS_CONFIRMED = 2;
    ORDER_STATUS_PROCESSING = 3;
    ORDER_STATUS_SHIPPED = 4;
    ORDER_STATUS_OUT_FOR_DELIVERY = 5;
    ORDER_STATUS_DELIVERED = 6;
    ORDER_STATUS_CANCELLED = 7;
    ORDER_STATUS_REFUNDED = 8;
    ORDER_STATUS_ON_HOLD = 9;
}

message OrderItem {
    string id = 1;
    string product_id = 2;
    string variant_id = 3;
    int32 quantity = 4;
    Money unit_price = 5;
    Money total_price = 6;
    string item_name = 7;
    map<string, string> attributes = 8;
}

message ShippingAddress {
    string recipient_name = 1;
    string address_line1 = 2;
    string address_line2 = 3;
    string city = 4;
    string state = 5;
    string postal_code = 6;
    string country = 7;
    string phone_number = 8;
    string instructions = 9;
}

message BillingAddress {
    string recipient_name = 1;
    string address_line1 = 2;
    string address_line2 = 3;
    string city = 4;
    string state = 5;
    string postal_code = 6;
    string country = 7;
    string phone_number = 8;
}

message PaymentInfo {
    string payment_method_id = 1;
    PaymentMethodType method_type = 2;
    string last_four_digits = 3;
    string card_brand = 4;
    google.protobuf.Timestamp expires_at = 5;
}

enum PaymentMethodType {
    PAYMENT_METHOD_TYPE_UNSPECIFIED = 0;
    PAYMENT_METHOD_TYPE_CREDIT_CARD = 1;
    PAYMENT_METHOD_TYPE_DEBIT_CARD = 2;
    PAYMENT_METHOD_TYPE_PAYPAL = 3;
    PAYMENT_METHOD_TYPE_BANK_TRANSFER = 4;
    PAYMENT_METHOD_TYPE_CRYPTO = 5;
    PAYMENT_METHOD_TYPE_GIFT_CARD = 6;
}

message OrderEvent {
    string event_id = 1;
    OrderStatus from_status = 2;
    OrderStatus to_status = 3;
    google.protobuf.Timestamp occurred_at = 4;
    string actor_id = 5;
    string reason = 6;
}

message CreateOrderRequest {
    string customer_id = 1;
    repeated CreateOrderItem items = 2;
    string shipping_address_id = 3;
    string billing_address_id = 4;
    string payment_method_id = 5;
    string promo_code = 6;
}

message CreateOrderItem {
    string product_id = 1;
    string variant_id = 2;
    int32 quantity = 3;
}

message GetOrderRequest {
    string order_id = 1;
}

message ListOrdersRequest {
    string customer_id = 1;
    repeated OrderStatus statuses = 2;
    google.protobuf.Timestamp start_date = 3;
    google.protobuf.Timestamp end_date = 4;
    int32 page_size = 5;
    string page_token = 6;
}

message ListOrdersResponse {
    repeated Order orders = 1;
    string next_page_token = 2;
    int32 total_count = 3;
}

message CancelOrderRequest {
    string order_id = 1;
    string reason = 2;
}

message StreamOrderUpdatesRequest {
    repeated string order_ids = 1;
    bool include_status_updates = 2;
    bool include_shipping_updates = 3;
}

message OrderUpdate {
    string order_id = 1;
    OrderUpdateType update_type = 2;
    Order order = 3;
    ShippingUpdate shipping_update = 4;
    google.protobuf.Timestamp timestamp = 5;
}

enum OrderUpdateType {
    ORDER_UPDATE_TYPE_UNSPECIFIED = 0;
    ORDER_UPDATE_TYPE_CREATED = 1;
    ORDER_UPDATE_TYPE_STATUS_CHANGED = 2;
    ORDER_UPDATE_TYPE_SHIPPED = 3;
    ORDER_UPDATE_TYPE_DELIVERED = 4;
    ORDER_UPDATE_TYPE_CANCELLED = 5;
}

message ShippingUpdate {
    string tracking_number = 1;
    string carrier = 2;
    OrderStatus status = 3;
    string location = 4;
    google.protobuf.Timestamp estimated_delivery = 5;
}

message UpdateOrderStatusRequest {
    string order_id = 1;
    OrderStatus new_status = 2;
    string reason = 3;
}
```

## 3. Streaming Patterns

### 3.1 Client Streaming Pattern

```protobuf
// Client sends multiple requests, server responds once
// Good for: file uploads, metric aggregation, batch processing

syntax = "proto3";

package analytics.v1;

import "google/protobuf/timestamp.proto";

service MetricsCollector {
    // Client streams metrics, server aggregates and responds
    rpc AggregateMetrics(stream MetricData) returns (AggregateMetricsResponse);
    
    // Client streams events, server acknowledges
    rpc RecordEvents(stream EventRecord) returns (RecordEventsResponse);
    
    // Client streams log entries, server streams acknowledgements
    rpc IngestLogs(stream LogEntry) returns (stream LogAcknowledgement);
}

message MetricData {
    string metric_name = 1;
    double value = 2;
    google.protobuf.Timestamp timestamp = 3;
    map<string, string> labels = 4;
    string source = 5;
}

message AggregateMetricsResponse {
    int64 processed_count = 1;
    AggregateResult aggregate = 2;
    repeated ProcessingWarning warnings = 3;
}

message AggregateResult {
    double sum = 1;
    double average = 2;
    double min = 3;
    double max = 4;
    double std_deviation = 5;
    int64 count = 6;
    google.protobuf.Timestamp window_start = 7;
    google.protobuf.Timestamp window_end = 8;
}

message ProcessingWarning {
    string metric_name = 1;
    string warning_code = 2;
    string warning_message = 3;
}

message EventRecord {
    string event_type = 1;
    string entity_id = 2;
    map<string, string> properties = 3;
    google.protobuf.Timestamp occurred_at = 4;
    string user_id = 5;
    string session_id = 6;
}

message RecordEventsResponse {
    int64 accepted_count = 1;
    int64 rejected_count = 2;
    repeated RejectionDetail rejections = 3;
}

message RejectionDetail {
    int32 index = 1;
    string reason = 2;
    string error_code = 3;
}

message LogEntry {
    string log_level = 1;
    string message = 2;
    string source_service = 3;
    string source_component = 4;
    string trace_id = 5;
    string span_id = 6;
    google.protobuf.Timestamp timestamp = 7;
    map<string, string> metadata = 8;
}

message LogAcknowledgement {
    int64 sequence_number = 1;
    bool success = 2;
    string message = 3;
    google.protobuf.Timestamp processed_at = 4;
}
```

### 3.2 Server Streaming Pattern

```protobuf
// Server sends multiple responses to single request
// Good for: notifications, live updates, data replication

syntax = "proto3";

package notification.v1;

import "google/protobuf/timestamp.proto";

service NotificationService {
    // Server streams notifications to client
    rpc SubscribeToNotifications(SubscribeRequest) returns (stream Notification);
    
    // Server streams price updates
    rpc SubscribeToPriceUpdates(PriceUpdateSubscription) returns (stream PriceUpdate);
    
    // Server streams order status updates
    rpc TrackOrderUpdates(TrackOrderRequest) returns (stream OrderStatusUpdate);
}

message SubscribeRequest {
    string user_id = 1;
    repeated NotificationChannel channels = 2;
    repeated string event_types = 3;
    NotificationFilter filter = 4;
}

enum NotificationChannel {
    NOTIFICATION_CHANNEL_UNSPECIFIED = 0;
    NOTIFICATION_CHANNEL_PUSH = 1;
    NOTIFICATION_CHANNEL_EMAIL = 2;
    NOTIFICATION_CHANNEL_SMS = 3;
    NOTIFICATION_CHANNEL_IN_APP = 4;
}

message NotificationFilter {
    int32 priority_minimum = 1;
    repeated string categories = 2;
    google.protobuf.Timestamp expires_after = 3;
}

message Notification {
    string notification_id = 1;
    string title = 2;
    string body = 3;
    NotificationPriority priority = 4;
    string category = 5;
    map<string, string> data = 6;
    google.protobuf.Timestamp created_at = 7;
    NotificationChannel channel = 8;
    bool requires_interaction = 9;
    string action_url = 10;
}

enum NotificationPriority {
    NOTIFICATION_PRIORITY_UNSPECIFIED = 0;
    NOTIFICATION_PRIORITY_LOW = 1;
    NOTIFICATION_PRIORITY_NORMAL = 2;
    NOTIFICATION_PRIORITY_HIGH = 3;
    NOTIFICATION_PRIORITY_URGENT = 4;
}

message PriceUpdateSubscription {
    repeated string product_ids = 1;
    repeated string category_ids = 2;
    PriceThreshold threshold = 3;
}

message PriceThreshold {
    string product_id = 1;
    double max_price = 2;
    double min_price = 3;
    bool notify_on_change = 4;
}

message PriceUpdate {
    string product_id = 1;
    Money previous_price = 2;
    Money new_price = 3;
    PriceChangeType change_type = 4;
    google.protobuf.Timestamp timestamp = 5;
}

enum PriceChangeType {
    PRICE_CHANGE_TYPE_UNSPECIFIED = 0;
    PRICE_CHANGE_TYPE_INCREASE = 1;
    PRICE_CHANGE_TYPE_DECREASE = 2;
    PRICE_CHANGE_TYPE_SET = 3;
}

message Money {
    string currency_code = 1;
    int64 amount = 2;
}

message TrackOrderRequest {
    string order_id = 1;
    repeated TrackingEventType event_types = 2;
}

enum TrackingEventType {
    TRACKING_EVENT_TYPE_UNSPECIFIED = 0;
    TRACKING_EVENT_TYPE_STATUS_CHANGE = 1;
    TRACKING_EVENT_TYPE_LOCATION_UPDATE = 2;
    TRACKING_EVENT_TYPE_DELIVERY_ATTEMPT = 3;
    TRACKING_EVENT_TYPE_DELIVERED = 4;
}

message OrderStatusUpdate {
    string order_id = 1;
    string event_type = 2;
    OrderStatus new_status = 3;
    google.protobuf.Timestamp timestamp = 4;
    OrderLocation location = 5;
    string description = 6;
}

enum OrderStatus {
    ORDER_STATUS_UNSPECIFIED = 0;
    ORDER_STATUS_PROCESSING = 1;
    ORDER_STATUS_SHIPPED = 2;
    ORDER_STATUS_IN_TRANSIT = 3;
    ORDER_STATUS_OUT_FOR_DELIVERY = 4;
    ORDER_STATUS_DELIVERED = 5;
    ORDER_STATUS_RETURNED = 6;
}

message OrderLocation {
    double latitude = 1;
    double longitude = 2;
    string address = 3;
    string city = 4;
    string state = 5;
    string postal_code = 6;
    string country = 7;
}
```

### 3.3 Bidirectional Streaming Pattern

```protobuf
// Both client and server stream messages
// Good for: chat, real-time collaboration, live queries

syntax = "proto3";

package collaboration.v1;

import "google/protobuf/timestamp.proto";

service DocumentCollaboration {
    // Real-time document editing
    rpc StreamDocumentChanges(stream DocumentChange) returns (stream DocumentChange);
    
    // Video call signaling
    rpc HandleVideoCall(stream VideoSignal) returns (stream VideoSignal);
    
    // Collaborative code editing
    rpc StreamCodeEdits(stream CodeEdit) returns (stream CodeEdit);
}

message DocumentChange {
    string document_id = 1;
    string session_id = 2;
    string user_id = 3;
    ChangeType change_type = 4;
    bytes change_data = 5;
    int32 version = 6;
    google.protobuf.Timestamp timestamp = 7;
    OperationContext context = 8;
}

enum ChangeType {
    CHANGE_TYPE_UNSPECIFIED = 0;
    CHANGE_TYPE_INSERT = 1;
    CHANGE_TYPE_DELETE = 2;
    CHANGE_TYPE_REPLACE = 3;
    CHANGE_TYPE_FORMAT = 4;
    CHANGE_TYPE_CURSOR_MOVE = 5;
    CHANGE_TYPE_SELECTION = 6;
}

message OperationContext {
    string cursor_position = 1;
    string selection_start = 2;
    string selection_end = 3;
    map<string, string> metadata = 4;
}

message VideoSignal {
    string call_id = 1;
    string participant_id = 2;
    SignalType signal_type = 3;
    bytes payload = 4;
    google.protobuf.Timestamp timestamp = 5;
}

enum SignalType {
    SIGNAL_TYPE_UNSPECIFIED = 0;
    SIGNAL_TYPE_OFFER = 1;
    SIGNAL_TYPE_ANSWER = 2;
    SIGNAL_TYPE_ICE_CANDIDATE = 3;
    SIGNAL_TYPE_MUTE = 4;
    SIGNAL_TYPE_UNMUTE = 5;
    SIGNAL_TYPE_VIDEO_ON = 6;
    SIGNAL_TYPE_VIDEO_OFF = 7;
    SIGNAL_TYPE_SCREEN_SHARE_START = 8;
    SIGNAL_TYPE_SCREEN_SHARE_STOP = 9;
    SIGNAL_TYPE_LEAVE = 10;
}

message CodeEdit {
    string document_id = 1;
    string session_id = 2;
    string user_id = 3;
    string user_name = 4;
    string user_color = 5;
    EditOperation operation = 6;
    TextRange range = 7;
    string new_text = 8;
    string old_text = 9;
    int32 version = 10;
    google.protobuf.Timestamp timestamp = 11;
    Language language = 12;
}

message EditOperation {
    OperationType type = 1;
    string description = 2;
}

enum OperationType {
    OPERATION_TYPE_UNSPECIFIED = 0;
    OPERATION_TYPE_INSERT = 1;
    OPERATION_TYPE_DELETE = 2;
    OPERATION_TYPE_REPLACE = 3;
    OPERATION_TYPE_RENAME = 4;
    OPERATION_TYPE_FORMAT = 5;
    OPERATION_TYPE_REFACTOR = 6;
}

message TextRange {
    int32 start_line = 1;
    int32 start_column = 2;
    int32 end_line = 3;
    int32 end_column = 4;
}

enum Language {
    LANGUAGE_UNSPECIFIED = 0;
    LANGUAGE_GO = 1;
    LANGUAGE_PYTHON = 2;
    LANGUAGE_TYPESCRIPT = 3;
    LANGUAGE_JAVA = 4;
    LANGUAGE_RUST = 5;
    LANGUAGE_CPP = 6;
}
```

## 4. Error Handling

### 4.1 Error Handling Patterns

```protobuf
syntax = "proto3";

package error.v1;

import "google/rpc/status.proto";
import "google/rpc/error_details.proto";

// Custom error service
service ErrorHandlingService {
    rpc DemonstrateErrors(DemoRequest) returns (DemoResponse);
}

message DemoRequest {
    ErrorScenario scenario = 1;
}

message DemoResponse {
    string result = 1;
}

// Error scenarios demonstrating best practices
enum ErrorScenario {
    ERROR_SCENARIO_UNSPECIFIED = 0;
    ERROR_SCENARIO_VALIDATION = 1;
    ERROR_SCENARIO_NOT_FOUND = 2;
    ERROR_SCENARIO_PERMISSION_DENIED = 3;
    ERROR_SCENARIO_ALREADY_EXISTS = 4;
    ERROR_SCENARIO_RATE_LIMITED = 5;
    ERROR_SCENARIO_INTERNAL = 6;
    ERROR_SCENARIO_UNAVAILABLE = 7;
}

// Recommended error code mappings
/*
┌─────────────────────────────────────────────────────────────────────────────┐
│                         gRPC Error Code Mappings                            │
├─────────────────────────────────────────────────────────────────────────────┤
│ gRPC Code          │ HTTP Code │ Use Case                                   │
├────────────────────┼───────────┼────────────────────────────────────────────┤
│ OK                 │ 200       │ Successful response                        │
│ INVALID_ARGUMENT   │ 400       │ Malformed request, validation errors       │
│ NOT_FOUND          │ 404       │ Resource doesn't exist                     │
│ ALREADY_EXISTS     │ 409       │ Conflict (duplicate key, etc.)             │
│ PERMISSION_DENIED  │ 403       │ Authenticated but not authorized           │
│ UNAUTHENTICATED    │ 401       │ Missing or invalid credentials            │
│ RESOURCE_EXHAUSTED │ 429       │ Rate limit exceeded                        │
│ FAILED_PRECONDITION│ 422       │ Prerequisites not met                      │
│ ABORTED            │ 409       │ Transaction aborted, concurrent modification│
│ OUT_OF_RANGE       │ 400       │ Invalid value for field                    │
│ UNIMPLEMENTED      │ 501       │ Method not implemented                     │
│ INTERNAL           │ 500       │ Unexpected server error                    │
│ UNAVAILABLE        │ 503       │ Service unavailable, retry later            │
│ DATA_LOSS          │ 500       │ Irrecoverable data loss                    │
└─────────────────────────────────────────────────────────────────────────────┘
*/
```

### 4.2 Error Detail Messages

```protobuf
// Structured error details for rich error handling

message DetailedError {
    string code = 1;
    string message = 2;
    repeated ErrorDetail details = 3;
    ErrorMetadata metadata = 4;
}

message ErrorDetail {
    string field = 1;
    string issue = 2;
    string value = 3;
    repeated string allowed_values = 4;
}

message ErrorMetadata {
    string request_id = 1;
    string service_name = 2;
    string method_name = 3;
    google.protobuf.Timestamp timestamp = 4;
    string environment = 5;
}

// Example Go error handling
/*
package main

import (
    "fmt"
    "google.golang.org/grpc/codes"
    "google.golang.org/grpc/status"
)

func handleGRPCError(err error) {
    s, ok := status.FromError(err)
    if !ok {
        // Not a gRPC error
        fmt.Printf("Non-gRPC error: %v\n", err)
        return
    }

    switch s.Code() {
    case codes.InvalidArgument:
        fmt.Printf("Validation error: %s\n", s.Message())
        for _, detail := range s.Details() {
            switch d := detail.(type) {
            case *errdetails.BadRequest:
                for _, violation := range d.FieldViolations {
                    fmt.Printf("  Field: %s, Error: %s\n",
                        violation.Field, violation.Description)
                }
            }
        }
    case codes.NotFound:
        fmt.Printf("Resource not found: %s\n", s.Message())
    case codes.PermissionDenied:
        fmt.Printf("Permission denied: %s\n", s.Message())
    case codes.ResourceExhausted:
        fmt.Printf("Rate limited: %s\n", s.Message())
        retryInfo, _ := s.Details().(*errdetails.RetryInfo)
        if retryInfo != nil {
            fmt.Printf("  Retry after: %v\n", retryInfo.RetryDelay)
        }
    case codes.Internal:
        fmt.Printf("Internal error: %s\n", s.Message())
    default:
        fmt.Printf("Unknown error: %s\n", s.Message())
    }
}
*/
```

## 5. Deadlines and Cancellation

### 5.1 Deadline Configuration

```protobuf
syntax = "proto3";

package deadline.v1;

import "google/protobuf/duration.proto";
import "google/protobuf/timestamp.proto";

service DeadlineService {
    rpc QuickOperation(QuickRequest) returns (QuickResponse);
    rpc MediumOperation(MediumRequest) returns (MediumResponse);
    rpc LongRunningOperation(LongRunningRequest) returns (LongRunningResponse);
    rpc StreamData(stream DataChunk) returns (stream DataChunk);
}

message QuickRequest {
    string data = 1;
}

message QuickResponse {
    string result = 1;
}

message MediumRequest {
    string data = 1;
}

message MediumResponse {
    string result = 1;
}

message LongRunningRequest {
    string task_id = 1;
}

message LongRunningResponse {
    string result = 1;
}

message DataChunk {
    bytes content = 1;
    int32 sequence = 2;
}

// Recommended timeout guidelines
/*
┌─────────────────────────────────────────────────────────────────────────────┐
│                      Timeout Recommendations                                │
├─────────────────────────────────────────────────────────────────────────────┤
│ Operation Type     │ Timeout Range     │ Rationale                          │
├────────────────────┼───────────────────┼────────────────────────────────────┤
│ Simple read        │ 100-500ms         │ Single DB query or cache hit        │
│ Complex read       │ 500ms-2s          │ Multiple queries, joins            │
│ Simple write       │ 200ms-1s          │ Single insert/update                │
│ Complex write      │ 1-5s              │ Transactions, multiple operations   │
│ Stream open        │ 5-10s             │ Connection establishment            │
│ Health check       │ 1-3s              │ Quick liveness check               │
│ Background job     │ No timeout        │ Use progress reporting instead      │
└─────────────────────────────────────────────────────────────────────────────┘

Recommended per-operation timeout annotations in proto:
- Use google.protobuf.Duration for explicit timeouts
- Set per-RPC timeouts in client code
- Use deadline propagation in service meshes
*/
```

### 5.2 Cancellation Patterns

```protobuf
// Cancellation support in service definitions

service CancellableService {
    // Long-running operation with cancellation support
    rpc ProcessLargeDataset(stream DataChunk) returns (ProcessResult);
    
    // Search with early termination
    rpc SearchWithTimeout(SearchRequest) returns (stream SearchResult);
}

message SearchRequest {
    string query = 1;
    int32 max_results = 2;
}

// Go cancellation example
/*
package main

import (
    "context"
    "fmt"
    "time"
    
    "google.golang.org/grpc"
    "google.golang.org/grpc/codes"
    "google.golang.org/grpc/status"
)

func callServiceWithCancellation(ctx context.Context, conn *grpc.ClientConn) error {
    client := NewServiceClient(conn)
    
    // Create a context with timeout
    ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
    defer cancel()
    
    // Call can be cancelled by client
    response, err := client.LongRunningOperation(ctx, &Request{})
    if err != nil {
        if st, ok := status.FromError(err); ok {
            if st.Code() == codes.Canceled {
                fmt.Println("Request was cancelled by client")
                return nil
            }
        }
        return err
    }
    
    return nil
}

// Server-side cancellation checking
func (s *Server) LongRunningOperation(
    req *Request,
    stream Service_LongRunningOperationServer,
) error {
    for {
        select {
        case <-stream.Context().Done():
            // Client disconnected or cancelled
            return stream.Context().Err()
        default:
            // Continue processing
        }
        
        // Do work chunk
        result, err := processChunk()
        if err != nil {
            return err
        }
        
        if err := stream.Send(result); err != nil {
            return err
        }
    }
}
*/
```

## 6. Complete .proto Files and Code

### 6.1 Full Production Service Example

```protobuf
// user_service.proto - Complete production-ready service definition

syntax = "proto3";

package user.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/duration.proto";
import "google/protobuf/empty.proto";
import "google/protobuf/wrappers.proto";
import "google/rpc/status.proto";
import "validate/validate.proto";
import "protoc-gen-openapiv2/options/annotations.proto";

option go_package = "github.com/example/user/v1;userpb";
option java_package = "com.example.user.v1";
option java_multiple_files = true;

// User management service
service UserService {
    // Create a new user
    rpc CreateUser(CreateUserRequest) returns (CreateUserResponse);
    
    // Get user by ID
    rpc GetUser(GetUserRequest) returns (GetUserResponse);
    
    // Update user
    rpc UpdateUser(UpdateUserRequest) returns (UpdateUserResponse);
    
    // Delete user (soft delete)
    rpc DeleteUser(DeleteUserRequest) returns (google.protobuf.Empty);
    
    // List users with pagination
    rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
    
    // Search users
    rpc SearchUsers(SearchUsersRequest) returns (SearchUsersResponse);
    
    // Batch get users
    rpc BatchGetUsers(BatchGetUsersRequest) returns (BatchGetUsersResponse);
    
    // Stream user updates
    rpc StreamUserUpdates(StreamUserUpdatesRequest) returns (stream UserUpdate);
}

message User {
    string id = 1 [(validate.rules).string.uuid = true];
    string email = 2 [(validate.rules).string.email = true];
    string display_name = 3 [(validate.rules).string.min_len = 1, (validate.rules).string.max_len = 100];
    UserRole role = 4;
    UserStatus status = 5;
    map<string, string> attributes = 6;
    google.protobuf.Timestamp created_at = 7;
    google.protobuf.Timestamp updated_at = 8;
    google.protobuf.Timestamp last_login_at = 9;
    bool email_verified = 10;
    string created_by = 11;
}

enum UserRole {
    USER_ROLE_UNSPECIFIED = 0;
    USER_ROLE_USER = 1;
    USER_ROLE_ADMIN = 2;
    USER_ROLE_SUPER_ADMIN = 3;
}

enum UserStatus {
    USER_STATUS_UNSPECIFIED = 0;
    USER_STATUS_ACTIVE = 1;
    USER_STATUS_INACTIVE = 2;
    USER_STATUS_SUSPENDED = 3;
    USER_STATUS_DELETED = 4;
}

message CreateUserRequest {
    string email = 1 [(validate.rules).string.email = true];
    string display_name = 2 [(validate.rules).string.min_len = 1];
    string password = 3 [(validate.rules).string.min_len = 8];
    UserRole role = 4;
    map<string, string> attributes = 5;
}

message CreateUserResponse {
    User user = 1;
    string verification_token = 2;
}

message GetUserRequest {
    string user_id = 1 [(validate.rules).string.uuid = true];
    repeated string fields = 2;
}

message GetUserResponse {
    User user = 1;
}

message UpdateUserRequest {
    string user_id = 1 [(validate.rules).string.uuid = true];
    string email = 2 [(validate.rules).string.email = true];
    string display_name = 3 [(validate.rules).string.min_len = 1];
    map<string, string> attributes = 4;
}

message UpdateUserResponse {
    User user = 1;
}

message DeleteUserRequest {
    string user_id = 1 [(validate.rules).string.uuid = true];
    string reason = 2;
}

message ListUsersRequest {
    UserRole role = 1;
    UserStatus status = 2;
    int32 page_size = 3 [(validate.rules).int32 = {gte: 1, lte: 100}];
    string page_token = 4;
    string order_by = 5;
}

message ListUsersResponse {
    repeated User users = 1;
    string next_page_token = 2;
    int32 total_count = 3;
}

message SearchUsersRequest {
    string query = 1;
    repeated UserRole roles = 2;
    repeated UserStatus statuses = 3;
    int32 page_size = 4 [(validate.rules).int32 = {gte: 1, lte: 100}];
    string page_token = 5;
}

message SearchUsersResponse {
    repeated User users = 1;
    repeated SearchFacet facets = 2;
    string next_page_token = 3;
    int32 total_count = 4;
}

message SearchFacet {
    string name = 1;
    repeated FacetValue values = 2;
}

message FacetValue {
    string value = 1;
    int32 count = 2;
}

message BatchGetUsersRequest {
    repeated string user_ids = 1 [(validate.rules).repeated.min_items = 1, (validate.rules).repeated.max_items = 100];
}

message BatchGetUsersResponse {
    repeated User users = 1;
    repeated NotFoundError not_found = 2;
}

message NotFoundError {
    string user_id = 1;
    string error = 2;
}

message StreamUserUpdatesRequest {
    repeated string user_ids = 1;
    bool include_profile_updates = 2;
    bool include_status_updates = 3;
}

message UserUpdate {
    string user_id = 1;
    UpdateType update_type = 2;
    User user = 3;
    google.protobuf.Timestamp timestamp = 4;
}

enum UpdateType {
    UPDATE_TYPE_UNSPECIFIED = 0;
    UPDATE_TYPE_CREATED = 1;
    UPDATE_TYPE_UPDATED = 2;
    UPDATE_TYPE_DELETED = 3;
    UPDATE_TYPE_STATUS_CHANGED = 4;
}
```

### 6.2 Go Server Implementation

```go
// server/main.go - Complete gRPC server implementation

package main

import (
    "context"
    "fmt"
    "log"
    "net"
    "sync"
    "time"

    "github.com/example/user/v1"
    "google.golang.org/grpc"
    "google.golang.org/grpc/codes"
    "google.golang.org/grpc/credentials"
    "google.golang.org/grpc/keepalive"
    "google.golang.org/grpc/metadata"
    "google.golang.org/grpc/peer"
    "google.golang.org/grpc/reflection"
    "google.golang.org/grpc/status"
    "google.golang.org/protobuf/types/known/emptypb"
    "google.golang.org/protobuf/types/known/timestamppb"
    "golang.org/x/sync/errgroup"
)

const (
    maxConcurrentStreams = 100
    maxRecvMsgSize       = 4 * 1024 * 1024 // 4MB
    maxSendMsgSize       = 4 * 1024 * 1024 // 4MB
)

type UserServer struct {
    userpb.UnimplementedUserServiceServer
    
    mu    sync.RWMutex
    users map[string]*userpb.User
    
    streamHub *StreamHub
}

type StreamHub struct {
    mu      sync.RWMutex
    streams map[string]map[string]chan *userpb.UserUpdate
}

func NewStreamHub() *StreamHub {
    return &StreamHub{
        streams: make(map[string]map[string]chan *userpb.UserUpdate),
    }
}

func (s *StreamHub) AddSubscriber(userID, streamID string, ch chan *userpb.UserUpdate) {
    s.mu.Lock()
    defer s.mu.Unlock()
    
    if s.streams[userID] == nil {
        s.streams[userID] = make(map[string]chan *userpb.UserUpdate)
    }
    s.streams[userID][streamID] = ch
}

func (s *StreamHub) RemoveSubscriber(userID, streamID string) {
    s.mu.Lock()
    defer s.mu.Unlock()
    
    if s.streams[userID] != nil {
        delete(s.streams[userID], streamID)
        if len(s.streams[userID]) == 0 {
            delete(s.streams, userID)
        }
    }
}

func (s *StreamHub) Broadcast(userID string, update *userpb.UserUpdate) {
    s.mu.RLock()
    defer s.mu.RUnlock()
    
    if streams, ok := s.streams[userID]; ok {
        for _, ch := range streams {
            select {
            case ch <- update:
            default:
                // Channel full, skip
            }
        }
    }
}

func NewUserServer() *UserServer {
    return &UserServer{
        users:      make(map[string]*userpb.User),
        streamHub: NewStreamHub(),
    }
}

func (s *UserServer) CreateUser(ctx context.Context, req *userpb.CreateUserRequest) (*userpb.CreateUserResponse, error) {
    // Extract metadata for logging
    md, _ := metadata.FromIncomingContext(ctx)
    log.Printf("CreateUser called by %v for email %s", md["user-id"], req.Email)
    
    // Validate request
    if req.Email == "" {
        return nil, status.Errorf(codes.InvalidArgument, "email is required")
    }
    if req.DisplayName == "" {
        return nil, status.Errorf(codes.InvalidArgument, "display_name is required")
    }
    if len(req.Password) < 8 {
        return nil, status.Errorf(codes.InvalidArgument, "password must be at least 8 characters")
    }
    
    // Check for existing user
    s.mu.RLock()
    for _, u := range s.users {
        if u.Email == req.Email {
            s.mu.RUnlock()
            return nil, status.Errorf(codes.AlreadyExists, "user with email %s already exists", req.Email)
        }
    }
    s.mu.RUnlock()
    
    // Generate ID and create user
    userID := generateUUID()
    now := timestamppb.Now()
    
    user := &userpb.User{
        Id:           userID,
        Email:        req.Email,
        DisplayName:  req.DisplayName,
        Role:         req.Role,
        Status:       userpb.UserStatus_USER_STATUS_ACTIVE,
        Attributes:   req.Attributes,
        CreatedAt:    now,
        UpdatedAt:    now,
        EmailVerified: false,
    }
    
    s.mu.Lock()
    s.users[userID] = user
    s.mu.Unlock()
    
    // Broadcast update
    s.streamHub.Broadcast(userID, &userpb.UserUpdate{
        UserId:      userID,
        UpdateType:  userpb.UpdateType_UPDATE_TYPE_CREATED,
        User:        user,
        Timestamp:   now,
    })
    
    return &userpb.CreateUserResponse{
        User:              user,
        VerificationToken: generateToken(),
    }, nil
}

func (s *UserServer) GetUser(ctx context.Context, req *userpb.GetUserRequest) (*userpb.GetUserResponse, error) {
    if req.UserId == "" {
        return nil, status.Errorf(codes.InvalidArgument, "user_id is required")
    }
    
    s.mu.RLock()
    user, ok := s.users[req.UserId]
    s.mu.RUnlock()
    
    if !ok {
        return nil, status.Errorf(codes.NotFound, "user %s not found", req.UserId)
    }
    
    // Handle partial response
    if len(req.Fields) > 0 {
        user = filterUserFields(user, req.Fields)
    }
    
    return &userpb.GetUserResponse{User: user}, nil
}

func (s *UserServer) UpdateUser(ctx context.Context, req *userpb.UpdateUserRequest) (*userpb.UpdateUserResponse, error) {
    if req.UserId == "" {
        return nil, status.Errorf(codes.InvalidArgument, "user_id is required")
    }
    
    s.mu.Lock()
    user, ok := s.users[req.UserId]
    if !ok {
        s.mu.Unlock()
        return nil, status.Errorf(codes.NotFound, "user %s not found", req.UserId)
    }
    
    // Update fields
    if req.Email != "" {
        user.Email = req.Email
    }
    if req.DisplayName != "" {
        user.DisplayName = req.DisplayName
    }
    if req.Attributes != nil {
        for k, v := range req.Attributes {
            user.Attributes[k] = v
        }
    }
    user.UpdatedAt = timestamppb.Now()
    
    s.users[req.UserId] = user
    s.mu.Unlock()
    
    // Broadcast update
    s.streamHub.Broadcast(req.UserId, &userpb.UserUpdate{
        UserId:     req.UserId,
        UpdateType: userpb.UpdateType_UPDATE_TYPE_UPDATED,
        User:       user,
        Timestamp:  user.UpdatedAt,
    })
    
    return &userpb.UpdateUserResponse{User: user}, nil
}

func (s *UserServer) DeleteUser(ctx context.Context, req *userpb.DeleteUserRequest) (*emptypb.Empty, error) {
    if req.UserId == "" {
        return nil, status.Errorf(codes.InvalidArgument, "user_id is required")
    }
    
    s.mu.Lock()
    user, ok := s.users[req.UserId]
    if !ok {
        s.mu.Unlock()
        return nil, status.Errorf(codes.NotFound, "user %s not found", req.UserId)
    }
    
    // Soft delete
    user.Status = userpb.UserStatus_USER_STATUS_DELETED
    user.UpdatedAt = timestamppb.Now()
    s.users[req.UserId] = user
    s.mu.Unlock()
    
    // Broadcast update
    s.streamHub.Broadcast(req.UserId, &userpb.UserUpdate{
        UserId:     req.UserId,
        UpdateType: userpb.UpdateType_UPDATE_TYPE_DELETED,
        User:       user,
        Timestamp:  user.UpdatedAt,
    })
    
    return &emptypb.Empty{}, nil
}

func (s *UserServer) ListUsers(req *userpb.ListUsersRequest, stream userpb.UserService_ListUsersServer) error {
    s.mu.RLock()
    defer s.mu.RUnlock()
    
    var users []*userpb.User
    for _, user := range s.users {
        if req.Role != userpb.UserRole_USER_ROLE_UNSPECIFIED && user.Role != req.Role {
            continue
        }
        if req.Status != userpb.UserStatus_USER_STATUS_UNSPECIFIED && user.Status != req.Status {
            continue
        }
        users = append(users, user)
    }
    
    // Send in batches
    batchSize := 10
    for i := 0; i < len(users); i += batchSize {
        end := i + batchSize
        if end > len(users) {
            end = len(users)
        }
        
        if err := stream.Send(&userpb.ListUsersResponse{
            Users:         users[i:end],
            NextPageToken: fmt.Sprintf("%d", end),
            TotalCount:    int32(len(users)),
        }); err != nil {
            return err
        }
    }
    
    return nil
}

func (s *UserServer) BatchGetUsers(ctx context.Context, req *userpb.BatchGetUsersRequest) (*userpb.BatchGetUsersResponse, error) {
    if len(req.UserIds) == 0 {
        return nil, status.Errorf(codes.InvalidArgument, "user_ids is required")
    }
    if len(req.UserIds) > 100 {
        return nil, status.Errorf(codes.InvalidArgument, "user_ids cannot exceed 100")
    }
    
    s.mu.RLock()
    defer s.mu.RUnlock()
    
    var users []*userpb.User
    var notFound []*userpb.NotFoundError
    
    for _, id := range req.UserIds {
        if user, ok := s.users[id]; ok {
            users = append(users, user)
        } else {
            notFound = append(notFound, &userpb.NotFoundError{
                UserId: id,
                Error:  "user not found",
            })
        }
    }
    
    return &userpb.BatchGetUsersResponse{
        Users:     users,
        NotFound:  notFound,
    }, nil
}

func (s *UserServer) StreamUserUpdates(req *userpb.StreamUserUpdatesRequest, stream userpb.UserService_StreamUserUpdatesServer) error {
    streamID := generateUUID()
    updateCh := make(chan *userpb.UserUpdate, 100)
    
    // Subscribe to updates for requested users
    for _, userID := range req.UserIds {
        s.streamHub.AddSubscriber(userID, streamID, updateCh)
    }
    
    defer func() {
        for _, userID := range req.UserIds {
            s.streamHub.RemoveSubscriber(userID, streamID)
        }
    }()
    
    // Stream updates to client
    for {
        select {
        case <-stream.Context().Done():
            return stream.Context().Err()
        case update := <-updateCh:
            // Filter updates based on request
            if req.IncludeProfileUpdates && update.UpdateType == userpb.UpdateType_UPDATE_TYPE_UPDATED {
                if err := stream.Send(update); err != nil {
                    return err
                }
            }
            if req.IncludeStatusUpdates && update.UpdateType == userpb.UpdateType_UPDATE_TYPE_STATUS_CHANGED {
                if err := stream.Send(update); err != nil {
                    return err
                }
            }
        }
    }
}

// Helper functions

func generateUUID() string {
    return fmt.Sprintf("%08x-%04x-%04x-%04x-%012x", 
        time.Now().UnixNano(),
        time.Now().Unix()%0xFFFF,
        0x4000 | (time.Now().UnixNano()>>48)&0x0FFF,
        0x8000 | (time.Now().UnixNano()>>32)&0x3FFF,
        time.Now().UnixNano(),
    )
}

func generateToken() string {
    b := make([]byte, 32)
    for i := range b {
        b[i] = byte(time.Now().UnixNano() % 256)
    }
    return fmt.Sprintf("%x", b)
}

func filterUserFields(user *userpb.User, fields []string) *userpb.User {
    // Implementation would filter user based on requested fields
    return user
}

// Server options

func withServerInterceptor() grpc.ServerOption {
    return grpc.UnaryInterceptor(func(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
        start := time.Now()
        
        // Extract caller info
        if p, ok := peer.FromContext(ctx); ok {
            log.Printf("Request from %s", p.Addr)
        }
        
        // Process request
        resp, err := handler(ctx, req)
        
        // Log completion
        log.Printf("Request %s completed in %v", info.FullMethod, time.Since(start))
        
        return resp, err
    })
}

func withStreamInterceptor() grpc.ServerOption {
    return grpc.StreamInterceptor(func(srv interface{}, ss grpc.ServerStream, info grpc.StreamServerInfo, handler grpc.ServerHandler) error {
        start := time.Now()
        
        wrapped := &wrappedServerStream{ServerStream: ss}
        err := handler(wrapped)
        
        log.Printf("Stream %s completed in %v", info.FullMethod, time.Since(start))
        
        return err
    })
}

type wrappedServerStream struct {
    grpc.ServerStream
}

func (w *wrappedServerStream) Context() context.Context {
    return context.WithValue(w.ServerStream.Context(), "start_time", time.Now())
}

// Main function

func main() {
    lis, err := net.Listen("tcp", ":50051")
    if err != nil {
        log.Fatalf("failed to listen: %v", err)
    }
    
    // Create credentials
    creds, err := credentials.newServerTLSFromFile("cert.pem", "key.pem")
    if err != nil {
        log.Fatalf("failed to create credentials: %v", err)
    }
    
    // Create server options
    opts := []grpc.ServerOption{
        grpc.Creds(creds),
        grpc.MaxConcurrentStreams(maxConcurrentStreams),
        grpc.MaxRecvMsgSize(maxRecvMsgSize),
        grpc.MaxSendMsgSize(maxSendMsgSize),
        withServerInterceptor(),
        withStreamInterceptor(),
        grpc.KeepaliveParams(keepalive.ServerParameters{
            MaxConnectionAge:      2 * time.Hour,
            MaxConnectionAgeGrace: 5 * time.Minute,
            Time:                  1 * time.Hour,
            Timeout:               20 * time.Second,
        }),
        grpc.KeepaliveEnforcementPolicy(keepalive.EnforcementPolicy{
            MinTime:             10 * time.Minute,
            PermitWithoutStream: true,
        }),
    }
    
    grpcServer := grpc.NewServer(opts...)
    
    // Register services
    userServer := NewUserServer()
    userpb.RegisterUserServiceServer(grpcServer, userServer)
    
    // Enable reflection for debugging
    reflection.Register(grpcServer)
    
    log.Println("Starting gRPC server on :50051")
    if err := grpcServer.Serve(lis); err != nil {
        log.Fatalf("failed to serve: %v", err)
    }
}
```

### 6.3 Go Client Implementation

```go
// client/main.go - Complete gRPC client implementation

package main

import (
    "context"
    "fmt"
    "log"
    "sync"
    "time"

    "github.com/example/user/v1"
    "google.golang.org/grpc"
    "google.golang.org/grpc/balancer"
    "google.golang.org/grpc/balancer/roundrobin"
    "google.golang.org/grpc/codes"
    "google.golang.org/grpc/credentials"
    "google.golang.org/grpc/encoding/gzip"
    "google.golang.org/grpc/metadata"
    "google.golang.org/grpc/status"
    "google.golang.org/protobuf/types/known/emptypb"
    "golang.org/x/oauth2"
)

const (
    maxRetries    = 3
    retryInterval = 1 * time.Second
)

type UserClient struct {
    conn   *grpc.ClientConn
    client userpb.UserServiceClient
    
    mu       sync.RWMutex
    token    string
    tokenTTL time.Time
}

func NewUserClient(ctx context.Context, endpoint string) (*UserClient, error) {
    // Load credentials
    creds, err := credentials.newTLS(
        &tls.Config{
            InsecureSkipVerify: false,
            MinVersion:         tls.VersionTLS12,
        },
    )
    if err != nil {
        return nil, fmt.Errorf("failed to load credentials: %w", err)
    }
    
    // Configure retry policy
    retryOpts := []grpc.CallOption{
        grpc.WaitForReady(true),
        grpc.retry grpc.Retry{
            Max: maxRetries,
            Backoff: grpc.ExponentialBackoff{
                Initial: retryInterval,
                Max:     10 * time.Second,
            },
        },
    }
    
    // Create connection with load balancing
    conn, err := grpc.DialContext(
        ctx,
        endpoint,
        grpc.WithTransportCredentials(creds),
        grpc.WithBalancerName(roundrobin.Name),
        grpc.WithDefaultServiceConfig(`{"loadBalancingPolicy":"round_robin"}`),
        grpc.WithUnaryInterceptor(UnaryClientInterceptor()),
        grpc.WithStreamInterceptor(StreamClientInterceptor()),
    )
    if err != nil {
        return nil, fmt.Errorf("failed to connect: %w", err)
    }
    
    return &UserClient{
        conn:   conn,
        client: userpb.NewUserServiceClient(conn),
    }, nil
}

func (c *UserClient) CreateUser(ctx context.Context, email, displayName, password string) (*userpb.User, error) {
    // Add auth metadata
    ctx, err := c.withAuth(ctx)
    if err != nil {
        return nil, err
    }
    
    resp, err := c.client.CreateUser(ctx, &userpb.CreateUserRequest{
        Email:       email,
        DisplayName: displayName,
        Password:    password,
        Role:        userpb.UserRole_USER_ROLE_USER,
    }, grpc.UseCompressor(gzip.Name))
    if err != nil {
        return nil, c.handleError(err)
    }
    
    return resp.User, nil
}

func (c *UserClient) GetUser(ctx context.Context, userID string) (*userpb.User, error) {
    ctx, err := c.withAuth(ctx)
    if err != nil {
        return nil, err
    }
    
    resp, err := c.client.GetUser(ctx, &userpb.GetUserRequest{
        UserId: userID,
    })
    if err != nil {
        return nil, c.handleError(err)
    }
    
    return resp.User, nil
}

func (c *UserClient) ListUsers(ctx context.Context, role userpb.UserRole, pageSize int32) ([]*userpb.User, error) {
    ctx, err := c.withAuth(ctx)
    if err != nil {
        return nil, err
    }
    
    var users []*userpb.User
    var nextToken string
    
    for {
        resp, err := c.client.ListUsers(ctx, &userpb.ListUsersRequest{
            Role:     role,
            PageSize: pageSize,
            PageToken: nextToken,
        })
        if err != nil {
            return nil, c.handleError(err)
        }
        
        users = append(users, resp.Users...)
        
        if resp.NextPageToken == "" {
            break
        }
        nextToken = resp.NextPageToken
    }
    
    return users, nil
}

func (c *UserClient) BatchGetUsers(ctx context.Context, userIDs []string) ([]*userpb.User, error) {
    ctx, err := c.withAuth(ctx)
    if err != nil {
        return nil, err
    }
    
    resp, err := c.client.BatchGetUsers(ctx, &userpb.BatchGetUsersRequest{
        UserIds: userIDs,
    })
    if err != nil {
        return nil, c.handleError(err)
    }
    
    if len(resp.NotFound) > 0 {
        log.Printf("Warning: %d users not found", len(resp.NotFound))
    }
    
    return resp.Users, nil
}

func (c *UserClient) StreamUserUpdates(ctx context.Context, userIDs []string) error {
    ctx, err := c.withAuth(ctx)
    if err != nil {
        return err
    }
    
    stream, err := c.client.StreamUserUpdates(ctx, &userpb.StreamUserUpdatesRequest{
        UserIds:              userIDs,
        IncludeProfileUpdates: true,
        IncludeStatusUpdates:  true,
    })
    if err != nil {
        return c.handleError(err)
    }
    
    for {
        update, err := stream.Recv()
        if err == io.EOF {
            return nil
        }
        if err != nil {
            return c.handleError(err)
        }
        
        log.Printf("Received update for user %s: %v", update.UserId, update.UpdateType)
    }
}

func (c *UserClient) withAuth(ctx context.Context) (context.Context, error) {
    c.mu.RLock()
    token := c.token
    expiry := c.tokenTTL
    c.mu.RUnlock()
    
    // Refresh token if needed
    if time.Now().After(expiry) {
        newToken, newExpiry, err := c.refreshToken(ctx)
        if err != nil {
            return nil, err
        }
        token = newToken
        expiry = newExpiry
        
        c.mu.Lock()
        c.token = newToken
        c.tokenTTL = newExpiry
        c.mu.Unlock()
    }
    
    // Add to metadata
    md := metadata.Pairs("authorization", "Bearer "+token)
    return metadata.NewOutgoingContext(ctx, md), nil
}

func (c *UserClient) refreshToken(ctx context.Context) (string, time.Time, error) {
    // OAuth token refresh logic
    return "token", time.Now().Add(time.Hour), nil
}

func (c *UserClient) handleError(err error) error {
    s, ok := status.FromError(err)
    if !ok {
        return fmt.Errorf("unknown error: %w", err)
    }
    
    switch s.Code() {
    case codes.Unavailable:
        return fmt.Errorf("service unavailable, retry later: %s", s.Message())
    case codes.NotFound:
        return fmt.Errorf("resource not found: %s", s.Message())
    case codes.PermissionDenied:
        return fmt.Errorf("permission denied: %s", s.Message())
    case codes.InvalidArgument:
        return fmt.Errorf("invalid argument: %s", s.Message())
    default:
        return fmt.Errorf("gRPC error %s: %s", s.Code(), s.Message())
    }
}

func (c *UserClient) Close() error {
    return c.conn.Close()
}

// Interceptors

func UnaryClientInterceptor() grpc.UnaryClientInterceptor {
    return func(ctx context.Context, method string, req, reply interface{}, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
        start := time.Now()
        
        // Add request ID
        reqID := uuid.New().String()
        ctx = metadata.AppendToOutgoingContext(ctx, "x-request-id", reqID)
        
        log.Printf("Sending request %s to %s", reqID, method)
        
        err := invoker(ctx, method, req, reply, cc, opts...)
        
        log.Printf("Request %s completed in %v with error: %v", reqID, time.Since(start), err)
        
        return err
    }
}

func StreamClientInterceptor() grpc.StreamClientInterceptor {
    return func(ctx context.Context, desc *grpc.StreamDesc, cc *grpc.ClientConn, method string, streamer grpc.Streamer, opts ...grpc.CallOption) (grpc.ClientStream, error) {
        reqID := uuid.New().String()
        ctx = metadata.AppendToOutgoingContext(ctx, "x-request-id", reqID)
        
        log.Printf("Starting stream %s to %s", reqID, method)
        
        stream, err := streamer(ctx, desc, cc, method, opts...)
        
        return &wrappedClientStream{stream, reqID}, err
    }
}

type wrappedClientStream struct {
    grpc.ClientStream
    reqID string
}

func (w *wrappedClientStream) RecvMsg(m interface{}) error {
    err := w.ClientStream.RecvMsg(m)
    if err != nil {
        log.Printf("Stream %s received error: %v", w.reqID, err)
    }
    return err
}

func (w *wrappedClientStream) SendMsg(m interface{}) error {
    err := w.ClientStream.SendMsg(m)
    if err != nil {
        log.Printf("Stream %s send error: %v", w.reqID, err)
    }
    return err
}
```

## 7. Decision Matrices

### 7.1 Protocol Selection Matrix

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              gRPC vs REST Selection Matrix                               │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Factor                        │ Use gRPC When          │ Use REST When                  │
├───────────────────────────────┼────────────────────────┼────────────────────────────────┤
│ Communication Pattern         │ Bidirectional/Streaming│ Request-Response only          │
│ Contract Requirements          │ Strong typing required │ Flexible schema acceptable     │
│ Code Generation               │ Strongly desired       │ Not critical                   │
│ Browser Support              │ Limited (needs wrapper)│ Native support                 │
│ Payload Size                 │ Small (~5-50KB)        │ Variable (can be large)        │
│ Performance                  │ Critical               │ Secondary                       │
│ Mobile Clients              │ Good for low bandwidth │ Universal support              │
│ Internal Services           │ Yes                    │ Consider OpenAPI               │
│ External/Public APIs        │ Rarely                 │ Common (REST preferred)        │
│ Polyglot Environments       │ Strong (good lib support)│ Strong                        │
│ Debugging/Testing          │ Harder                 │ Easier (curl, browser)         │
├───────────────────────────────┴────────────────────────┴────────────────────────────────┤
│ Recommended: Use gRPC for internal service-to-service communication, especially        │
│ when streaming is needed, performance is critical, or strong typing provides value.    │
│ Use REST for external APIs, browser clients, or when simplicity trumps performance.    │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Streaming Pattern Selection

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           Streaming Pattern Selection Matrix                            │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Pattern              │ Use When                          │ Don't Use When              │
├─────────────────────┼──────────────────────────────────┼─────────────────────────────┤
│ Server Streaming    │ - Live dashboards                 │ - Need response before send │
│                     │ - Notifications                   │ - Short request/response    │
│                     │ - Log streaming                   │ - Fire-and-forget            │
│                     │ - Price/position updates          │ - Connection unstable        │
├─────────────────────┼──────────────────────────────────┼─────────────────────────────┤
│ Client Streaming    │ - File upload                     │ - Need response immediately │
│                     │ - Metric aggregation              │ - Few messages              │
│                     │ - Batch processing                │ - Server can't track state  │
│                     │ - Sensor data collection          │ - Order matters              │
├─────────────────────┼──────────────────────────────────┼─────────────────────────────┤
│ Bidirectional       │ - Chat applications                │ - Simple request/response   │
│                     │ - Real-time collaboration         │ - One-way data flow         │
│                     │ - Game state sync                 │ - Connection unreliable      │
│                     │ - Live queries                    │ - Need request ordering      │
└─────────────────────┴──────────────────────────────────┴─────────────────────────────┘
```

### 7.3 Error Handling Matrix

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              gRPC Error Code Selection Matrix                            │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Code                  │ HTTP │ When to Use                      │ Response Handling       │
├───────────────────────┼──────┼──────────────────────────────────┼─────────────────────────┤
│ OK                    │ 200  │ Successful operation              │ Return response         │
├───────────────────────┼──────┼──────────────────────────────────┼─────────────────────────┤
│ INVALID_ARGUMENT      │ 400  │ - Malformed request syntax        │ Show user error, fix    │
│                       │      │ - Validation failed               │ and retry               │
│                       │      │ - Unknown field                  │                         │
├───────────────────────┼──────┼──────────────────────────────────┼─────────────────────────┤
│ NOT_FOUND            │ 404  │ - Resource doesn't exist          │ Return 404, suggest     │
│                       │      │ - ID references deleted resource  │ alternatives if possible│
├───────────────────────┼──────┼──────────────────────────────────┼─────────────────────────┤
│ ALREADY_EXISTS       │ 409  │ - Duplicate key                   │ Return conflict error   │
│                       │      │ - Resource with same unique field │ and existing resource  │
├───────────────────────┼──────┼──────────────────────────────────┼─────────────────────────┤
│ PERMISSION_DENIED    │ 403  │ - Authenticated but not authorized │ Return 403, no retry    │
│                       │      │ - Insufficient role/scope        │ until permissions change│
├───────────────────────┼──────┼──────────────────────────────────┼─────────────────────────┤
│ UNAUTHENTICATED      │ 401  │ - No credentials                  │ Prompt for login,       │
│                       │      │ - Expired/invalid token           │ refresh and retry       │
├───────────────────────┼──────┼──────────────────────────────────┼─────────────────────────┤
│ RESOURCE_EXHAUSTED   │ 429  │ - Rate limit exceeded             │ Return 429, Retry-After │
│                       │      │ - Quota exceeded                 │ header, backoff and retry│
├───────────────────────┼──────┼──────────────────────────────────┼─────────────────────────┤
│ FAILED_PRECONDITION  │ 422  │ - Prerequisites not met           │ Don't retry, fix        │
│                       │      │ - Operation not valid in state    │ prerequisites first     │
├───────────────────────┼──────┼──────────────────────────────────┼─────────────────────────┤
│ ABORTED              │ 409  │ - Transaction conflict            │ Retry with backoff      │
│                       │      │ - Concurrent modification         │ or new transaction      │
├───────────────────────┼──────┼──────────────────────────────────┼─────────────────────────┤
│ INTERNAL            │ 500  │ - Unexpected server error         │ Log, alert, don't       │
│                       │      │ - Unhandled exception             │ expose details to client│
├───────────────────────┼──────┼──────────────────────────────────┼─────────────────────────┤
│ UNAVAILABLE         │ 503  │ - Service down                    │ Retry with backoff      │
│                       │      │ - Temporary overload              │ using exponential delay │
└───────────────────────┴──────┴──────────────────────────────────┴─────────────────────────┘
```

## 8. Anti-Patterns

### 8.1 Common gRPC Anti-Patterns

```markdown
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              gRPC Anti-Patterns to Avoid                                │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Anti-Pattern                    │ Problem                       │ Solution                │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ Using proto2 syntax            │ Missing features, larger msgs  │ Use proto3 always       │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ Using complex types in maps     │ Limited language support       │ Use repeated messages   │
│                                 │ for complex map values        │ with key field instead  │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ Deep nesting in messages       │ Deserialization overhead       │ Flatten or use one-of   │
│                                 │ Hard to version               │ for alternatives        │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ No versioning strategy         │ Breaking changes impossible    │ Version in package      │
│                                 │                               │ name (v1, v2, etc)      │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ Large messages > 1MB           │ Memory pressure                │ Use chunking/streaming  │
│                                 │ Streaming issues              │ or pagination           │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ No deadline propagation        │ Requests run forever           │ Always propagate ctx    │
│                                 │ Resource leaks                │ deadlines              │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ Ignoring stream context       │ Streams hang after client      │ Check ctx.Done()       │
│                                 │ disconnect                     │ in all streaming RPCs   │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ No retry logic                │ Transient failures kill ops    │ Use gRPC retry policy   │
│                                 │                               │ with backoff            │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ Using bytes for structured    │ No schema validation           │ Use proper message      │
│ data                           │ Can't inspect/debug            │ types                  │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ Missing error details         │ Poor client error handling     │ Always include Status   │
│                                 │ Generic errors to users        │ with error details     │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ Over-using streaming          │ Complex to implement           │ Use unary unless        │
│                                 │ Hard to debug                  │ streaming adds value    │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ No connection pooling        │ Connection overhead             │ Use channel pools      │
│                                 │ Latency on each call           │ for high-throughput    │
├─────────────────────────────────┼───────────────────────────────┼─────────────────────────┤
│ Ignoring backpressure        │ Memory exhaustion              │ Implement flow control  │
│                                 │ OOM on slow consumers          │ in streaming scenarios  │
└─────────────────────────────────┴───────────────────────────────┴─────────────────────────┘
```

### 8.2 Bad vs Good Examples

```protobuf
// BAD: Deeply nested message
message BadProduct {
    Category category = 1;  // Complex nested type
    Vendor vendor = 2;     // Another complex type
    repeated Review reviews = 3;  // List of complex types
    
    message Category {
        string id = 1;
        string name = 2;
        ParentCategory parent = 3;  // Recursive!
        repeated Category children = 4;  // More recursion!
    }
}

// GOOD: Flat structure with references
message GoodProduct {
    string id = 1;
    string name = 2;
    string category_id = 3;
    string vendor_id = 4;
    repeated string review_ids = 5;
}

// BAD: Using maps for complex values
message BadOrder {
    map<string, OrderItem> items = 1;  // Map with message value
    map<string, Discount> discounts = 2;  // Another complex map
}

// GOOD: Using repeated messages with key fields
message GoodOrder {
    repeated OrderItem items = 1;
    repeated Discount discounts = 2;
}

message OrderItem {
    string sku = 1;
    int32 quantity = 2;
    int64 price_cents = 3;
}

// BAD: No versioning in package
package myservice;  // No version!

// GOOD: Version in package
package myservice.v1;
package myservice.v2;
```

## 9. Best Practices Summary

### 9.1 Proto Design Best Practices

```markdown
1. Always Use Proto3
   - Simpler syntax, better defaults
   - No required fields (use validation instead)
   - Better JSON mapping

2. Package Naming
   - Use full domain + service + version: `com.example.service.v1`
   - Makes routing and code generation cleaner

3. Message Naming
   - Use CamelCase for messages and enums
   - Use descriptive names: GetUserRequest not GetUserReq
   - Singular for single items, plural for repeated

4. Field Naming
   - Use snake_case: `user_id` not `userId`
   - Be consistent across all messages
   - Use clear names: `created_at` not `ct`

5. Field Numbers
   - Reserve 1-15 for frequently used fields
   - Don't reuse field numbers
   - Document field meaning when non-obvious

6. Enums
   - Prefix with message name: `OrderStatus`
   - First value should be UNSPECIFIED = 0
   - Use explicit values, not implicit

7. OneOf Usage
   - Great for mutually exclusive fields
   - Reduces null checks
   - Cleaner than optional fields
```

### 9.2 Service Design Best Practices

```markdown
1. RPC Naming
   - Verb-Noun pattern: GetUser, CreateOrder
   - List for collections: ListUsers
   - Stream prefix for streaming: StreamUpdates

2. Method Semantics
   - Idempotent methods for GET-like operations
   - Non-idempotent for CREATE (use POST)
   - Use proper HTTP mapping for REST compatibility

3. Streaming
   - Only use when it adds value
   - Implement proper backpressure
   - Handle connection drops gracefully

4. Error Handling
   - Map to appropriate gRPC codes
   - Include error details for debugging
   - Never expose internal details

5. Deadline Propagation
   - Always pass context with deadline
   - Use reasonable defaults
   - Handle deadline exceeded gracefully
```

### 9.3 Production Checklist

```
Pre-Production Checklist:
□ Proto files validated with protoc
□ Generated code compiles for all target languages
□ Service documentation generated
□ OpenAPI spec exported for REST compatibility
□ Error codes documented
□ Retry policies configured
□ Timeout values set appropriately
□ Health check endpoint implemented
□ Metrics and tracing configured
□ Load testing completed
□ Failover testing completed
□ Security review completed
```

---

## Links

### Official Documentation
- [Protocol Buffers Language Guide](https://developers.google.com/protocol-buffers/docs/proto3)
- [gRPC Core Concepts](https://grpc.io/docs/what-is-grpc/core-concepts/)
- [gRPC Authentication](https://grpc.io/docs/guides/auth/)
- [gRPC Error Handling](https://grpc.io/docs/guides/error/)
- [gRPC Status Codes](https://github.com/grpc/grpc/blob/master/doc/statuscodes.md)

### Protocol Buffer Tools
- [protoc Installation](https://github.com/protocolbuffers/protobuf/releases)
- [grpc-web](https://github.com/grpc/grpc-web)
- [protoc-gen-doc](https://github.com/pseudomuto/protoc-gen-doc)
- [buf Schema Management](https://buf.build/)
- [grpcio-tools](https://pypi.org/project/grpcio-tools/)

### Language-Specific
- [Go gRPC](https://github.com/grpc/grpc-go)
- [Java gRPC](https://github.com/grpc/grpc-java)
- [Python gRPC](https://github.com/grpc/grpc/tree/master/src/python/grpcio)
- [Node.js gRPC](https://github.com/grpc/grpc-node)
- [C++ gRPC](https://github.com/grpc/grpc/tree/master/src/cpp)

### gRPC Ecosystem
- [gRPC Gateway](https://github.com/grpc-ecosystem/grpc-gateway)
- [gRPC UI](https://github.com/fullstorydev/grpcui)
- [grpcurl](https://github.com/fullstorydev/grpcurl)
- [BloomRPC](https://github.com/bloomberg/bloomrpc)
- [gRPC喵](https://github.com/grpc-ecosystem/grpc-spring-boot-starter)

### Validation
- [validate extension](https://github.com/bufbuild/protoc-gen-validate)
- [ protobuf validation patterns](https://github.com/envoyproxy/protobuf-validation)

### Best Practices
- [Google API Design Guide](https://cloud.google.com/apis/design)
- [Uber Protobuf Style Guide](https://github.com/uber/prototool)
- [Yelp gRPC Examples](https://github.com/Yelp/grpc-elixir)