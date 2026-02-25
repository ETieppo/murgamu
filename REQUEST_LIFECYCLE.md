segue o lifecycle da little nightmare

Criado pra ter uma visão macro do fluxo onde será possível, e manualmente encontrar possiveis erros e principalmente reconhecer áreas de risco e implementação de testes, compreendo que isso não substitui boas ferramentas de benchmarks como k6, vgrant e por ai vai, algumas mais simples(como as citadas) já foram usadas e as demais ainda estão planejadas, no momento ainda não sei como aplicar todos os testes necessários e isso causa falso negativo, portanto o que me resta é paciência, dei destaque a algumas coisas que a ia gerou.

# AI GENERATED FILE!
# Murgamü — Ciclo de Vida Completo de uma Requisição POST

> Documento gerado por análise estática do código-fonte.
> Cada etapa referencia o arquivo e as linhas exatas onde acontece.

---

## Visão Geral (Resumo Visual)

```
Código do usuário                          Compilação (macros)
─────────────────                          ──────────────────
MurServer::new()                           #[module]   → impl MurModule
  .inject(DbPool)                          #[service]  → impl MurService + MurServiceFactory
  .module(UserModule)                      #[controller("/users")] → impl MurController + MurControllerFactory
  .add_global_guard(AuthGuard)             #[post("/")]  → MurRouteDefinition { method, path, handler }
  .add_global_interceptor(LogInterceptor)
  .add_middleware(CorsMiddleware)          !!! NUNCA EXECUTADO !!!
  .add_exception_filter(GlobalFilter)
  .bind("0.0.0.0:3000")?                   ← monta tudo aqui
__.run().await                             ← abre socket TCP
|
|
|
|--|>		┌─────────────────────────────────────────────────────────────────-┐
    		│                    REQUISIÇÃO POST CHEGA                         │
    		│                                                                  │
    		│  TCP accept → TokioIo::new(stream) → hyper http1::Builder      │
    		│       → service_fn  → router.handle(req)                       │
    		│                                                                  │
    		│  handle():                                                       │
    		│    1. req.into_parts() → (Parts, Incoming body)                 │
    		│    2. method = "POST", path = "/users"                           │
    		│    3. find_route("POST", "/users")                               │
    		│       └─ itera routes_by_method["POST"]                          │
    		│          └─ pattern.match_path("/users") → Some(path_params)    │
    		│    4. collect_body(body) → Option<Bytes>  ⚠ SEM LIMITE          │
    		│    5. MurRequestContext::new(parts, body, params, container)     │
    		│                                                                  │
    		│  execute_handler(route, ctx):                                    │
    		│    6. ctx.with_access_control(route.access_control)              │
    		│    7. global_guards  → guard.can_activate(&ctx)                 │
    		│    8. route.guards   → guard.can_activate(&ctx)                 │
    		│    9. global_interceptors → interceptor.before(&ctx)            │
    		│   10. route.interceptors  → interceptor.before(&ctx)            │
    		│   11. (route.handler)(ctx.clone()).await   ← CLOSURE DO MACRO   │
    		│       └─ extrai MurJson<T>, Param<T>, MurQuery<T> etc do ctx     │
    		│       └─ controller.create_user(json, param...).await            │
    		│          └─ self.user_service.do_something()                     │
    		│          └─ retorna MurRes (Ok(Response) ou Err(MurError))       │
    		│   12. route.interceptors.rev() → interceptor.after(&ctx, res)   │
    		│   13. global_interceptors.rev() → interceptor.after(&ctx, res)  │
    		│   14. Se Err(e) → handle_error(e) → exception_filters / default│
    		│   15. Response<Full<Bytes>> retorna para hyper                   │
    		└─────────────────────────────────────────────────────────────────-┘
```

---

## FASE 1 — Build Time (Macros Proc-Macro)

Antes de qualquer execução, os macros `#[controller]`, `#[service]`, `#[module]` geram
código em tempo de compilação. Entender isso é essencial porque define **como** os
handlers são montados.

---

### 1.1 `#[service]` — Transforma struct em MurService

**Arquivo:** `macros/src/service/generate_service_impl.rs`

Dado:

```rust
#[service]
struct UserService {
    db: Arc<DbPool>,
}
```

O macro gera:

```rust
// impl MurService → permite ser armazenado no container
impl MurService for UserService {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

// impl MurServiceFactory → permite criação automática com DI
impl MurServiceFactory for UserService {
    fn create(_injects: &MurInjects, _container: &MurServiceContainer) -> Self {
        // Campo `db: Arc<DbPool>` → detectado automaticamente
        let __mur_inject_0 = _container.get_required::<DbPool>();
        Self::new(__mur_inject_0)
    }
}

// impl MurDependencies → declara de quais TypeId depende
impl MurDependencies for UserService {
    fn dependencies() -> &'static [std::any::TypeId] {
        static DEPS: OnceLock<Vec<TypeId>> = OnceLock::new();
        DEPS.get_or_init(|| vec![TypeId::of::<DbPool>()]).as_slice()
    }
}
```

**Detalhes da inferência de injeção** (`generate_service_impl.rs`):

- Campo `Arc<T>` → `_container.get_required::<T>()` (obrigatório)
- Campo `Option<Arc<T>>` → `_container.get::<T>()` (opcional)
- Se `#[service(injects = [...])]` for usado, usa lista explícita em vez de inferir

---

### 1.2 `#[controller("/users")]` — Transforma impl block em MurController

**Arquivo:** `macros/src/controller/controller_impl.rs`

Dado:

```rust
#[controller("/users")]
impl UserController {
    pub fn new(user_service: Arc<UserService>) -> Self { ... }

    #[post("/")]
    #[public]
    pub async fn create_user(&self, body: MurJson<CreateUserDto>) -> MurRes { ... }
}
```

O macro gera **três implementações**:

#### 1.2.1 `impl MurController` — define rotas

```rust
impl MurController for UserController {
    fn routes(
        self: Arc<Self>,
        container: Arc<MurServiceContainer>,
    ) -> Vec<MurRouteDefinition> {
        let mut routes = Vec::new();
        let controller = self.clone();

        // Para cada método anotado com #[post("/")]:
        routes.push(MurRouteDefinition {
            method: "POST".to_string(),
            path: "/users".to_string(),  // base_path + route_path
            handler: {
                let controller_clone = controller.clone();
                Arc::new(move |ctx: MurRequestContext| -> MurFuture {
                    let controller = controller_clone.clone();
                    Box::pin(async move {
                        // ← EXTRAÇÃO DE PARÂMETROS GERADA PELO MACRO
                        let body: MurJson<CreateUserDto> = match ctx.json() {
                            Ok(data) => MurJson(data),
                            Err(e) => {
                                return MurResponder::error(
                                    &format!("Failed to parse JSON: {}", e)
                                );
                            }
                        };
                        // ← CHAMADA DO MÉTODO DO CONTROLLER
                        controller.create_user(body).await
                    })
                }) as MurRouteHandler
            },
            is_public: true,       // por causa do #[public]
            allowed_roles: vec![], // sem #[role(...)]
        });

        routes
    }

    fn base_path(&self) -> &str { "/users" }
}
```

**Arquivo:** `macros/src/controller/generate_handler.rs`

A extração de parâmetros é decidida por tipo (`macros/src/utils/analyze_parameter.rs`):

| Tipo no método                 | ParamKind       | Código gerado                                    |
| ------------------------------ | --------------- | ------------------------------------------------ |
| `MurRequestContext` / `MurReq` | `Context`       | `let x = ctx.clone();`                           |
| `MurJson<T>`                   | `Json(T)`       | `ctx.json()` → deserializa body como JSON        |
| `MurQuery<T>`                  | `Query(T)`      | `serde_urlencoded::from_str(uri.query())`        |
| `MurPath<T>`                   | `Path(T)`       | `serde_json::from_value(path_params)`            |
| `Param<T>`                     | `Param(T)`      | `ctx.path_params.get("name")` → `T::from_str`    |
| `MurBody`                      | `Body`          | `ctx.body.clone()` (Bytes raw)                   |
| `MurText`                      | `Text`          | `String::from_utf8(ctx.body)`                    |
| `MurHeader`                    | `Header`        | `MurHeader(String::new())` ⚠ sempre vazio        |
| `Option<Bytes>`                | `RawBody`       | `ctx.body.clone()`                               |
| `Arc<MurServiceContainer>`     | `Container`     | `ctx.container.clone()`                          |
| `Parts`                        | `Parts`         | `ctx.parts.clone()`                              |
| qualquer outro `T`             | `CustomJson(T)` | `ctx.json::<T>()` (tenta deserializar como JSON) |

#### 1.2.2 `impl MurControllerFactory` — construtor com DI

```rust
impl MurControllerFactory for UserController {
    fn create(injects: &MurInjects, _container: &MurServiceContainer) -> Self {
        // `new(user_service: Arc<UserService>)` → Arc<T> detectado
        let __mur_ctor_0 = _container.get_required::<UserService>();
        Self::new(__mur_ctor_0)
    }
}
```

#### 1.2.3 `impl MurDependencies`

```rust
impl MurDependencies for UserController {
    fn dependencies() -> &'static [TypeId] {
        static DEPS: OnceLock<Vec<TypeId>> = OnceLock::new();
        DEPS.get_or_init(|| vec![TypeId::of::<UserService>()]).as_slice()
    }
}
```

---

### 1.3 `#[module]` — Transforma struct em MurModule

**Arquivo:** `macros/src/module.rs`

Dado:

```rust
#[module(
    imports = [AuthModule],
    controllers = [UserController],
    services = [UserService],
    exports = [UserService],
)]
struct UserModule;
```

O macro gera:

```rust
struct UserModule {
    user_service: OnceLock<Arc<UserService>>,  // lazy init
}

impl UserModule {
    pub fn new() -> Self {
        Self { user_service: OnceLock::new() }
    }

    // Helper para criar service com cache (OnceLock)
    fn __mur_service_user_service(
        &self,
        _injects: &MurInjects,
        _container: &MurServiceContainer,
    ) -> Arc<UserService> {
        self.user_service
            .get_or_init(|| {
                Arc::new(<UserService as MurServiceFactory>::create(_injects, _container))
            })
            .clone()
    }
}

impl MurModule for UserModule {
    fn name(&self) -> &str { "UserModule" }

    fn imports(&self) -> Vec<Arc<dyn MurModule>> {
        vec![Arc::new(AuthModule::new()) as Arc<dyn MurModule>]
    }

    fn exports(&self) -> Vec<TypeId> {
        vec![TypeId::of::<UserService>()]
    }

    fn controllers_with_injects(
        &self,
        _injects: &MurInjects,
        _container: &MurServiceContainer,
    ) -> Vec<Arc<dyn MurController>> {
        vec![
            Arc::new(
                <UserController as MurControllerFactory>::create(_injects, _container)
            ) as Arc<dyn MurController>
        ]
    }

    fn services_with_injects(
        &self,
        _injects: &MurInjects,
        _container: &MurServiceContainer,
    ) -> Vec<(TypeId, Arc<dyn MurService>)> {
        // Resolução topológica de dependências entre services do módulo
        let mut pending = vec![
            (TypeId::of::<UserService>(), UserService::dependencies(), maker_fn)
        ];
        // Loop: cria services na ordem correta de dependência
        // Se ciclo detectado → panic!
        ...
    }
}
```

A resolução topológica no `services_with_injects` garante que se `ServiceA` depende de
`ServiceB`, `ServiceB` é criado primeiro e registrado no container local antes de
`ServiceA` ser criado.

---

## FASE 2 — Inicialização do Servidor (Runtime)

### 2.1 `MurServer::new()` → Construção

**Arquivo:** `src/server/builder.rs` linhas 33-48

```
MurServer {
    modules: Vec<Box<dyn MurModule>>         ← vazio
    container: MurServiceContainer            ← vazio (HashMap com capacidade 32)
    injects: MurInjects                       ← vazio (HashMap com capacidade 8)
    guards: Vec<Box<dyn MurGuard>>            ← vazio
    interceptors: Vec<Box<dyn MurInterceptor>>← vazio
    middleware: Vec<Box<dyn MurMiddleware>>    ← vazio
    exception_filters: Vec<Box<dyn MurExceptionFilter>> ← vazio
    config: MurServerConfig::default()        ← addr=127.0.0.1:3000, body_limit=2MB, etc
    on_startup: Vec<Box<dyn Fn()>>            ← vazio
    on_shutdown: Vec<Box<dyn Fn()>>           ← vazio
}
```

---

### 2.2 Métodos builder (encadeamento)

**Arquivo:** `src/server/builder.rs`

```
server
  .inject(db_pool)                    → injects.register(db_pool)       // linha 110
                                        // HashMap<TypeId, Arc<dyn MurInjectable>>
                                        // TypeId::of::<DbPool>() → Arc::new(db_pool)

  .module(UserModule)                 → modules.push(Box::new(module))  // linha 104

  .add_global_guard(AuthGuard)        → guards.push(Box::new(guard))    // linha 118

  .add_global_interceptor(Logger)     → interceptors.push(Box::new(i))  // linha 122

  .add_middleware(MurCors::new())     → middleware.push(Box::new(mw))   // linha 126
                                        // ⚠ COLETADO MAS NUNCA PASSADO AO ROUTER

  .add_exception_filter(Filter)      → exception_filters.push(Box::new(f)) // linha 130

  .service(SomeGlobalService)        → container.register(service)      // linha 134
                                        // HashMap<TypeId, Arc<dyn MurService>>

  .on_startup(|| println!("up!"))    → on_startup.push(Box::new(hook)) // linha 167

  .on_shutdown(|| println!("down!")) → on_shutdown.push(Box::new(hook))// linha 171
```

---

### 2.3 `.bind("0.0.0.0:3000")` → Montagem Final

**Arquivo:** `src/server/builder.rs` linhas 175-283

Esta é a fase mais complexa. Passo a passo:

```
bind("0.0.0.0:3000")
  │
  ├─ 1. Resolve endereço: addr.to_socket_addrs()?.next()
  │     → chama bind_addr(addr)
  │
  ├─ 2. self.config.addr = addr
  │
  ├─ 3. self.injects.on_init()
  │     → para cada injectable registrado, chama inject.on_init()
  │       (src/server/service/injects.rs linha 52)
  │
  ├─ 4. Cria container global base:
  │     let mut app_global = MurServiceContainer::new();
  │     app_global.merge(self.container);
  │     // Agora app_global tem os services registrados com .service()
  │
  ├─ 5. let mut runtime_global = app_global.clone();
  │
  ├─ 6. Para CADA módulo em self.modules:
  │     │
  │     ├─ 6a. module.on_init()
  │     │
  │     ├─ 6b. Resolve imports/exports recursivamente:
  │     │       build_imports_exports_container(module, injects, app_global)
  │     │       │
  │     │       │  Para cada módulo importado (recursivo com visited set):
  │     │       │    fill_exports_rec_arc(imported, injects, out, visited)
  │     │       │      → cria services do módulo importado
  │     │       │      → filtra apenas os que estão em exports()
  │     │       │      → registra no container "visible"
  │     │       │
  │     │       └─ retorna container "visible" com exports dos imports
  │     │
  │     ├─ 6c. visible.merge(app_global.clone())
  │     │       // visible agora = exports dos imports + services globais
  │     │
  │     ├─ 6d. Cria services locais do módulo:
  │     │       module.services_with_injects(&injects, &visible)
  │     │       // Macro #[module] gerou resolução topológica
  │     │       // Cada service é criado via MurServiceFactory::create(injects, container)
  │     │       // Exemplo: UserService::create(injects, visible)
  │     │       //   → visible.get_required::<DbPool>()
  │     │       //   → UserService::new(db_pool_arc)
  │     │
  │     ├─ 6e. Registra services locais no visible:
  │     │       for (tid, svc) in local_services {
  │     │           visible.register_dyn_with_id(tid, svc);
  │     │       }
  │     │
  │     └─ 6f. runtime_global.merge(visible)
  │            // runtime_global acumula TUDO de todos os módulos
  │
  ├─ 7. let container = Arc::new(runtime_global);
  │     // Container final imutável compartilhado por todas as requests
  │
  ├─ 8. let mut router = MurRouter::new(Arc::clone(&container));
  │     // Cria router com HashMap pré-alocado para 7 métodos HTTP
  │     // (src/server/router/core.rs linhas 36-53)
  │
  ├─ 9. Registra guards globais no router:
  │     for guard in self.guards {
  │         router.add_guard_boxed(guard.create(&self.injects));
  │         // guard.create() é trait MurGuard::create(), NÃO MurGuardFactory
  │         // → router.global_guards.push(Arc::from(guard))
  │     }
  │
  ├─ 10. Registra interceptors globais no router:
  │      for interceptor in self.interceptors {
  │          router.add_interceptor_boxed(interceptor);
  │          // → router.global_interceptors.push(Arc::from(interceptor))
  │      }
  │
  ├─ 11. Registra exception filters no router:
  │      for filter in self.exception_filters {
  │          router.exception_filters.push(Arc::from(filter));
  │      }
  │
  ├─ 12. ⚠ MIDDLEWARE IGNORADO — self.middleware NUNCA é passado ao router
  │      // O campo router.global_middleware permanece vazio
  │      // MurCors, MurRateLimit, MurSecurityHeaders etc nunca funcionam
  │
  ├─ 13. Para CADA módulo, registra controllers:
  │      for module in &self.modules {
  │          println!("Loading module: {}", module.name());  // se logging
  │          for controller in module.controllers_with_injects(&injects, &container) {
  │              router.register_controller(controller);
  │          }
  │      }
  │      │
  │      └─ register_controller(controller):
  │         │  (src/server/router/core.rs linhas 56-87)
  │         │
  │         ├─ controller.routes(container)
  │         │  // Chama o código gerado pelo macro #[controller]
  │         │  // Retorna Vec<MurRouteDefinition>
  │         │  // Cada MurRouteDefinition = { method, path, handler, is_public, allowed_roles }
  │         │
  │         ├─ Para cada route_def:
  │         │   ├─ MurRoutePattern::new(&route_def.path)
  │         │   │  // Parseia "/users/:id" em segments:
  │         │   │  //   [Literal("users"), Param("id")]
  │         │   │  // Calcula specificity: Literal=+100, Param=+10, Wildcard=+1, CatchAll=-100
  │         │   │
  │         │   ├─ MurRouteEntry::new(pattern, handler)
  │         │   │  // Entrada com: pattern, handler, guards=[], interceptors=[], metadata={}
  │         │   │
  │         │   ├─ entry.access_control = MurRouteAccessControl {
  │         │   │      is_public: route_def.is_public,
  │         │   │      allowed_roles: route_def.allowed_roles,
  │         │   │  }
  │         │   │
  │         │   └─ routes_by_method["POST"].push(entry)
  │         │
  │         └─ sort_all_routes()
  │            // Ordena por specificity descendente
  │            // Rotas mais específicas ("/users/profile") ficam antes de genéricas ("/users/:id")
  │
  ├─ 14. Se logging habilitado: router.print_routes()
  │      // Imprime o papiro ASCII com todas as rotas registradas
  │
  ├─ 15. Se TLS configurado: MurTlsAcceptor::new(tls_config)
  │      // Constrói ServerConfig do rustls
  │      // ⚠ Ignora min_version, max_version, client_auth
  │
  └─ 16. Retorna MurServerRunner {
             router: Arc::new(router),
             config,
             modules,
             injects,
             on_startup,
             on_shutdown,
             tls_acceptor,
         }
```

---

### 2.4 `.run().await` → Inicia Loop TCP

**Arquivo:** `src/server/runner.rs` linhas 30-58

```
run()
  │
  ├─ 1. Executa hooks on_startup:
  │     for hook in &self.on_startup { hook(); }
  │
  ├─ 2. let listener = TcpListener::bind(self.config.addr).await?;
  │
  ├─ 3. println!("Murgamü server listening on http://127.0.0.1:3000");
  │
  └─ 4. Decide qual loop usar (4 variantes):
        │
        ├─ TLS + graceful_shutdown   → run_tls_with_graceful_shutdown(listener)
        ├─ TLS + sem graceful        → run_tls_forever(listener)
        ├─ sem TLS + graceful        → run_with_graceful_shutdown(listener)
        └─ sem TLS + sem graceful    → run_forever(listener)
            // Default: sem TLS + graceful (graceful_shutdown=true por default)
```

---

## FASE 3 — Uma Requisição POST Chega

### Cenário: `POST /users` com body JSON `{"name":"João","email":"joao@test.com"}`

---

### 3.1 Aceitação TCP e Hyper

**Arquivo:** `src/server/runner.rs` linhas 60-90 (run_forever como exemplo)

```
loop {
    │
    ├─ 1. let (stream, _remote_addr) = listener.accept().await?;
    │     // Socket TCP aceito. _remote_addr disponível mas DESCARTADO.
    │     // ⚠ Nenhum IP real salvo — client_ip() depende de headers forjáveis
    │
    ├─ 2. let io = TokioIo::new(stream);
    │     // Wrapper do tokio para hyper
    │
    ├─ 3. let router = Arc::clone(&self.router);
    │     // Clone barato do Arc
    │
    └─ 4. tokio::task::spawn(async move {
           // Nova task assíncrona para esta conexão
           │
           ├─ 5. let service = service_fn(move |req| {
           │         let router = Arc::clone(&router);
           │         async move { router.handle(req).await }
           │     });
           │     // Cria um hyper Service que delega para router.handle()
           │
           └─ 6. http1::Builder::new()
                    .serve_connection(io, service)
                    .with_upgrades()
                    .await
                 // ⚠ Sempre HTTP/1.1, nunca HTTP/2
                 // ⚠ Sem keep_alive_timeout configurado
                 // ⚠ Sem body size limit no nível hyper
                 //
                 // Hyper parseia os bytes TCP:
                 //   - Lê request line: "POST /users HTTP/1.1\r\n"
                 //   - Lê headers: "Content-Type: application/json\r\n..."
                 //   - Prepara body stream (Incoming)
                 //   - Cria Request<Incoming>
                 //   - Chama service_fn → router.handle(req)
       });
}
```

---

### 3.2 `router.handle(req)` — Ponto de Entrada do Router

**Arquivo:** `src/server/router/core.rs` linhas 318-340

```
pub async fn handle(&self, req: Request<Incoming>) -> MurRes
  │
  ├─ 1. let (parts, body) = req.into_parts();
  │     //
  │     // parts: http::request::Parts {
  │     //   method: POST,
  │     //   uri: /users,
  │     //   version: HTTP/1.1,
  │     //   headers: HeaderMap {
  │     //     "content-type": "application/json",
  │     //     "host": "localhost:3000",
  │     //     "content-length": "45",
  │     //     ...
  │     //   },
  │     //   extensions: Extensions,
  │     // }
  │     //
  │     // body: hyper::body::Incoming (stream, ainda não lido)
  │
  ├─ 2. let method = parts.method.to_string().to_uppercase();
  │     // method = "POST"
  │
  ├─ 3. let path = parts.uri.path();
  │     // path = "/users"
  │
  ├─ 4. let route_match = self.find_route(&method, path);
  │     │
  │     │  find_route("POST", "/users"):
  │     │  (src/server/router/core.rs linhas 404-412)
  │     │    │
  │     │    ├─ let routes = self.routes_by_method.get("POST")?;
  │     │    │  // Vec<MurRouteEntry> ordenado por specificity desc
  │     │    │
  │     │    └─ for route in routes {
  │     │           if let Some(params) = route.pattern.match_path("/users") {
  │     │               return Some((route, params));
  │     │           }
  │     │       }
  │     │
  │     │  match_path("/users"):
  │     │  (src/server/router/pattern.rs linhas 67-74)
  │     │    │
  │     │    ├─ Se is_static (rota sem :param, *, **):
  │     │    │    normalize_path("/users") == self.pattern ?
  │     │    │    "/users" == "/users" → Some(HashMap::with_capacity(0))
  │     │    │
  │     │    └─ Se não é static → match_path_dynamic()
  │     │         Itera segmentos do pattern vs segmentos do path:
  │     │           Literal("users") vs "users" → match
  │     │           Param("id") vs "123" → params.insert("id", "123")
  │     │           Wildcard vs qualquer → consome 1 segmento
  │     │           CatchAll → consome todos os segmentos restantes
  │     │
  │     │  Retorna: Some((&MurRouteEntry, HashMap<String,String>))
  │     │           ou None se nenhuma rota matches
  │     │
  │     └─ route_match = Some((route, {}))
  │        // path_params vazio pois "/users" é rota estática
  │
  ├─ 5. BRANCH: route_match.is_none() ?
  │     │
  │     ├─ Se NONE e method == "OPTIONS":
  │     │     return self.handle_options(path)
  │     │     // Retorna 204 com Allow header
  │     │
  │     ├─ Se NONE e method == "HEAD":
  │     │     Tenta find_route("GET", path)
  │     │     Se encontrou → processa como GET
  │     │
  │     ├─ Se NONE (nenhum match):
  │     │     return self.handle_not_found(path)
  │     │     // ⚠ Mesmo se not_found_handler setado, NUNCA é chamado (bug)
  │     │     // Retorna: 404 {"error":"Not Found","message":"No route found for path: /xxx"}
  │     │
  │     └─ Se SOME: continua ↓
  │
  ├─ 6. let (route, path_params) = route_match.unwrap();
  │     // route = &MurRouteEntry { pattern, handler, guards, interceptors, access_control }
  │     // path_params = HashMap::new()  (vazio para rota estática "/users")
  │
  ├─ 7. let body_bytes = self.collect_body(body).await?;
  │     │
  │     │  collect_body(body):
  │     │  (src/server/router/core.rs linhas 341-352)
  │     │    │
  │     │    ├─ body.collect().await
  │     │    │  // Hyper lê TODOS os bytes do stream TCP para memória
  │     │    │  // ⚠ SEM NENHUM LIMITE DE TAMANHO
  │     │    │  // config.body_limit (2MB) existe mas NUNCA é verificado aqui
  │     │    │
  │     │    ├─ let bytes = collected.to_bytes();
  │     │    │
  │     │    └─ if bytes.is_empty() { Ok(None) } else { Ok(Some(bytes)) }
  │     │       // Para nosso POST com JSON: Some(Bytes) com o conteúdo
  │     │
  │     │  Se erro ao ler body → Err(MurError::Hyper(e))
  │     │  → Retorna erro imediatamente, nunca chega ao handler
  │     │
  │     └─ body_bytes = Some(b'{"name":"João","email":"joao@test.com"}')
  │
  ├─ 8. Monta o contexto da requisição:
  │     let ctx = MurRequestContext::new(
  │         parts,           // method, uri, headers, extensions
  │         body_bytes,      // Some(Bytes)
  │         path_params,     // HashMap<String,String>
  │         Arc::clone(&self.container)  // container de services compartilhado
  │     );
  │     │
  │     │  MurRequestContext::new():
  │     │  (src/server/http/request.rs linhas 25-36)
  │     │    │
  │     │    └─ MurRequestContext {
  │     │           parts,
  │     │           body: Some(Bytes),
  │     │           path_params: {},
  │     │           container: Arc<MurServiceContainer>,
  │     │           query_cache: OnceLock::new(),    // lazy, calculado no primeiro acesso
  │     │           access_control: None,            // setado no próximo passo
  │     │       }
  │
  └─ 9. return self.execute_handler(route, ctx).await;
        // ↓ detalhe completo abaixo
```

---

### 3.3 `execute_handler(route, ctx)` — Pipeline de Execução

**Arquivo:** `src/server/router/core.rs` linhas 354-401

```
execute_handler(route: &MurRouteEntry, ctx: MurRequestContext) -> MurRes
  │
  ├─ PASSO 1: Anexa access control ao contexto
  │
  │  let ctx = ctx.with_access_control(route.access_control.clone());
  │  // (src/server/http/request.rs linhas 238-241)
  │  //
  │  // ctx.access_control = Some(MurRouteAccessControl {
  │  //     is_public: true,         // #[public] no handler
  │  //     allowed_roles: {},       // sem #[role(...)]
  │  // })
  │  //
  │  // Isso permite que guards consultem:
  │  //   ctx.is_public_route()    → true
  │  //   ctx.allowed_roles()      → Some(&HashSet{})
  │  //   ctx.has_allowed_role("admin") → true (vazio = todos permitidos)
  │
  │
  ├─ PASSO 2: Guards Globais
  │
  │  for guard in &self.global_guards {
  │      if !guard.can_activate(&ctx).await {
  │          return guard.rejection_response();
  │      }
  │  }
  │  │
  │  │  Exemplo: AuthGuard.can_activate(&ctx):
  │  │  (trait MurGuard em src/server/guard/contract.rs)
  │  │    │
  │  │    ├─ Recebe &MurRequestContext
  │  │    │  // O guard pode consultar:
  │  │    │  //   ctx.is_public_route()    → pular validação se true
  │  │    │  //   ctx.bearer_token()       → extrair JWT do header Authorization
  │  │    │  //   ctx.header("X-Api-Key")  → checar API key
  │  │    │  //   ctx.has_allowed_role(r)  → checar role
  │  │    │  //   ctx.service::<T>()       → acessar services do container
  │  │    │
  │  │    ├─ Retorna bool:
  │  │    │    true  → prossegue para o próximo guard
  │  │    │    false → PARA AQUI, retorna rejection_response()
  │  │    │
  │  │    └─ rejection_response() default:
  │  │         403 Forbidden {"error":"Forbidden","message":"Access denied!"}
  │  │         // Pode ser sobrescrito na implementação do guard
  │  │
  │  │  Guards são executados em ORDEM de registro.
  │  │  Se QUALQUER guard rejeita → resposta imediata, nada mais executa.
  │  │
  │  │  No nosso cenário: rota é #[public], guard típico verifica
  │  │  ctx.is_public_route() e retorna true → aprovado.
  │
  │
  ├─ PASSO 3: Guards da Rota (route-level)
  │
  │  for guard in &route.guards {
  │      if !guard.can_activate(&ctx).await {
  │          return guard.rejection_response();
  │      }
  │  }
  │  │
  │  │  Guards específicos da rota. Atualmente, as rotas registradas
  │  │  via register_controller() sempre têm route.guards = []
  │  │  (src/server/router/entry.rs linha 22).
  │  │  Apenas rotas com entry.guards populados manualmente teriam guards aqui.
  │  │
  │  └─ No nosso cenário: vazio, pula.
  │
  │
  ├─ PASSO 4: Interceptors Globais — BEFORE
  │
  │  for interceptor in &self.global_interceptors {
  │      if let Err(e) = interceptor.before(&ctx).await {
  │          return self.handle_error(e);
  │      }
  │  }
  │  │
  │  │  Exemplo: LogInterceptor.before(&ctx):
  │  │  (trait MurInterceptor em src/server/interceptor/contract.rs)
  │  │    │
  │  │    ├─ Recebe &MurRequestContext (imutável)
  │  │    │  // Pode ler headers, method, path, body, etc.
  │  │    │  // Pode fazer logging, métricas, validação cross-cutting
  │  │    │
  │  │    ├─ Retorna Result<(), MurError>:
  │  │    │    Ok(())  → prossegue
  │  │    │    Err(e)  → PARA, vai para handle_error(e)
  │  │    │
  │  │    └─ Default impl: Box::pin(async { Ok(()) })
  │  │       // Se não sobrescrito, é no-op
  │  │
  │  └─ No nosso cenário: LogInterceptor faz println e retorna Ok(()).
  │
  │
  ├─ PASSO 5: Interceptors da Rota — BEFORE
  │
  │  for interceptor in &route.interceptors {
  │      if let Err(e) = interceptor.before(&ctx).await {
  │          return self.handle_error(e);
  │      }
  │  }
  │  │
  │  │  Mesma lógica dos globais, mas específicos da rota.
  │  │  Rotas registradas via register_controller() têm route.interceptors = [].
  │  │
  │  └─ No nosso cenário: vazio, pula.
  │
  │
  ├─ PASSO 6: EXECUÇÃO DO HANDLER ★★★
  │
  │  let result = (route.handler)(ctx.clone()).await;
  │  │
  │  │  route.handler é o MurRouteHandler (Arc<dyn Fn(MurRequestContext) -> MurFuture>)
  │  │  gerado pelo macro #[controller] em tempo de compilação.
  │  │
  │  │  Desdobramento do que acontece dentro da closure:
  │  │  (gerado por macros/src/controller/generate_handler.rs)
  │  │
  │  │  Arc::new(move |ctx: MurRequestContext| -> MurFuture {
  │  │      let controller = controller_clone.clone();  // Arc<UserController>
  │  │      Box::pin(async move {
  │  │          │
  │  │          ├─ EXTRAÇÃO DE PARÂMETROS
  │  │          │
  │  │          │  Para `body: MurJson<CreateUserDto>`:
  │  │          │    let body: MurJson<CreateUserDto> = match ctx.json() {
  │  │          │        Ok(data) => MurJson(data),
  │  │          │        Err(e) => {
  │  │          │            return MurResponder::error(&format!("Failed to parse JSON: {}", e));
  │  │          │            // → 400 Bad Request com mensagem de erro
  │  │          │        }
  │  │          │    };
  │  │          │    │
  │  │          │    │  ctx.json::<CreateUserDto>():
  │  │          │    │  (src/server/http/request.rs linhas 194-201)
  │  │          │    │    │
  │  │          │    │    ├─ let body = self.body.as_ref()
  │  │          │    │    │    .ok_or(MurError::BadRequest("Missing request body"))?;
  │  │          │    │    │
  │  │          │    │    └─ serde_json::from_slice::<CreateUserDto>(body)
  │  │          │    │         .map_err(|e| MurError::BadRequest(format!("Invalid JSON: {}", e)))
  │  │          │    │
  │  │          │    │  Se body ausente → Err(BadRequest)
  │  │          │    │  Se JSON inválido → Err(BadRequest)
  │  │          │    │  Se campo faltando no DTO → Err(BadRequest)
  │  │          │    │  Se ok → Ok(CreateUserDto { name: "João", email: "joao@test.com" })
  │  │          │    │
  │  │          │    └─ body = MurJson(CreateUserDto { name: "João", email: "joao@test.com" })
  │  │          │
  │  │          │  (Outros tipos de parâmetro, se existissem:)
  │  │          │
  │  │          │  Para `id: Param<i64>` em rota "/users/:id":
  │  │          │    ctx.path_params.get("id") → Some("123")
  │  │          │    i64::from_str("123") → Ok(123)
  │  │          │    id = Param(123)
  │  │          │
  │  │          │  Para `query: MurQuery<PaginationDto>`:
  │  │          │    ctx.parts.uri.query() → Some("page=1&limit=10")
  │  │          │    serde_urlencoded::from_str("page=1&limit=10")
  │  │          │    query = MurQuery(PaginationDto { page: 1, limit: 10 })
  │  │          │
  │  │          │  Para `ctx: MurRequestContext`:
  │  │          │    ctx = ctx.clone()  (acesso total ao contexto)
  │  │          │
  │  │          │
  │  │          ├─ CHAMADA DO MÉTODO DO CONTROLLER
  │  │          │
  │  │          │  controller.create_user(body).await
  │  │          │  │
  │  │          │  │  Dentro do controller (código do usuário):
  │  │          │  │
  │  │          │  │  pub async fn create_user(&self, body: MurJson<CreateUserDto>) -> MurRes {
  │  │          │  │      │
  │  │          │  │      ├─ Acessa o service injetado:
  │  │          │  │      │    self.user_service  // Arc<UserService>
  │  │          │  │      │    // Foi injetado no construtor via MurControllerFactory::create()
  │  │          │  │      │    // que chamou _container.get_required::<UserService>()
  │  │          │  │      │
  │  │          │  │      ├─ Chama o service:
  │  │          │  │      │    let user = self.user_service.create(body.into_inner()).await?;
  │  │          │  │      │    │
  │  │          │  │      │    │  Dentro do service (código do usuário):
  │  │          │  │      │    │    pub async fn create(&self, dto: CreateUserDto) -> Result<User, MurError> {
  │  │          │  │      │    │        // Usa o inject (ex: pool de banco):
  │  │          │  │      │    │        //   self.db  // Arc<DbPool>
  │  │          │  │      │    │        //   injetado via MurServiceFactory::create()
  │  │          │  │      │    │        //   que chamou _container.get_required::<DbPool>()
  │  │          │  │      │    │        let conn = self.db.get()?;
  │  │          │  │      │    │        // insere no banco...
  │  │          │  │      │    │        Ok(user)
  │  │          │  │      │    │    }
  │  │          │  │      │    │
  │  │          │  │      │    └─ Retorna Result<User, MurError>
  │  │          │  │      │       // O ? propaga MurError se falhar
  │  │          │  │      │
  │  │          │  │      └─ Constrói a resposta:
  │  │          │  │           MurHttpResponse::created().json(user)
  │  │          │  │           │
  │  │          │  │           │  MurHttpResponse::created():
  │  │          │  │           │  (src/server/http/response/http_response.rs linha 8)
  │  │          │  │           │    → MurResponseBuilder::new().status(StatusCode::CREATED)
  │  │          │  │           │
  │  │          │  │           │  .json(user):
  │  │          │  │           │  (src/server/http/response/builder.rs linhas 36-50)
  │  │          │  │           │    ├─ serde_json::to_string(&user)
  │  │          │  │           │    │  → '{"id":1,"name":"João","email":"joao@test.com"}'
  │  │          │  │           │    ├─ Response::builder()
  │  │          │  │           │    │    .status(201)
  │  │          │  │           │    │    .header("Content-Type", "application/json")
  │  │          │  │           │    │    .body(Full::new(Bytes::from(json_string)))
  │  │          │  │           │    └─ Ok(Response<Full<Bytes>>)
  │  │          │  │           │
  │  │          │  │           └─ MurRes = Ok(Response { status: 201, headers: {content-type: application/json}, body: ... })
  │  │          │  │
  │  │          │  └─ Retorna MurRes
  │  │          │
  │  │          └─ A closure retorna MurRes
  │  │      })
  │  │  })
  │  │
  │  └─ result: MurRes = Ok(Response { status: 201, ... })
  │
  │
  ├─ PASSO 7: let mut response = result;
  │
  │
  ├─ PASSO 8: Interceptors da Rota — AFTER (ordem REVERSA)
  │
  │  for interceptor in route.interceptors.iter().rev() {
  │      response = interceptor.after(&ctx, response).await;
  │  }
  │  │
  │  │  interceptor.after(&ctx, response):
  │  │  (trait MurInterceptor em src/server/interceptor/contract.rs)
  │  │    │
  │  │    ├─ Recebe &MurRequestContext + MurRes (a response do handler)
  │  │    │  // Pode modificar a response: adicionar headers, transformar body, etc.
  │  │    │  // Pode substituir completamente a response
  │  │    │
  │  │    └─ Retorna MurRes (potencialmente modificada)
  │  │
  │  │  Nota: ordem reversa — se interceptors foram A, B, C na ida (before),
  │  │  na volta (after) são C, B, A. Padrão onion/middleware.
  │  │
  │  └─ No nosso cenário: vazio, pula.
  │
  │
  ├─ PASSO 9: Interceptors Globais — AFTER (ordem REVERSA)
  │
  │  for interceptor in self.global_interceptors.iter().rev() {
  │      response = interceptor.after(&ctx, response).await;
  │  }
  │  │
  │  │  Exemplo: LogInterceptor.after(&ctx, response):
  │  │    // Pode logar status code, tempo de resposta, etc.
  │  │    // Pode adicionar headers como X-Request-Id
  │  │    // Retorna response (possivelmente modificada)
  │  │
  │  └─ No nosso cenário: LogInterceptor loga e retorna response sem modificar.
  │
  │
  └─ PASSO 10: Tratamento Final de Erros

     match response {
         Ok(res) => Ok(res),     // ← Sucesso! Retorna a Response para hyper
         Err(e) => self.handle_error(e),
     }
     │
     │  Se a response é Ok → retorna diretamente
     │
     │  Se a response é Err(MurError) → handle_error(e):
     │  (src/server/router/core.rs linhas 414-432)
     │    │
     │    ├─ 1. Tenta exception filters (em ordem):
     │    │     for filter in &self.exception_filters {
     │    │         if filter.can_handle(&error) {
     │    │             return filter.catch(error, &ctx);
     │    │         }
     │    │     }
     │    │     │
     │    │     │  filter.can_handle(&error):
     │    │     │  (src/server/error/contract.rs)
     │    │     │    Default: retorna true para qualquer erro
     │    │     │    Pode ser sobrescrito para filtrar por tipo de erro
     │    │     │
     │    │     │  filter.catch(error, &ctx):
     │    │     │    Transforma o MurError em MurRes personalizada
     │    │     │    Ex: formatar erros de validação, esconder detalhes internos
     │    │     │
     │    │     └─ Se algum filter pode tratar → retorna a response do filter
     │    │
     │    ├─ 2. Se nenhum filter tratou, tenta error_handler customizado:
     │    │     if let Some(handler) = &self.error_handler {
     │    │         return handler(error);
     │    │     }
     │    │
     │    └─ 3. Fallback: error.into_response()
     │         (src/server/error/builder.rs linhas 144-162)
     │           │
     │           ├─ let status = error.status_code();
     │           │  // MurError::BadRequest → 400
     │           │  // MurError::NotFound → 404
     │           │  // MurError::Internal → 500
     │           │  // etc.
     │           │
     │           └─ Response JSON:
     │              {
     │                "error": "Bad request: Invalid JSON: missing field `name`",
     │                "status": 400,
     │                "kind": "bad_request"
     │              }
     │              // ⚠ Expõe mensagem interna completa
     │
     └─ No nosso cenário: response é Ok → retorna 201 Created.
```

---

### 3.4 Retorno da Response para o Cliente

```
router.handle(req) retorna MurRes = Ok(Response { status: 201, ... })
  │
  ├─ Volta para service_fn no runner.rs
  │  // service_fn(move |req| { router.handle(req) })
  │
  ├─ hyper http1::Builder recebe a Response<Full<Bytes>>
  │  // Serializa em HTTP/1.1:
  │  //   HTTP/1.1 201 Created\r\n
  │  //   Content-Type: application/json\r\n
  │  //   Content-Length: 52\r\n
  │  //   \r\n
  │  //   {"id":1,"name":"João","email":"joao@test.com"}
  │
  ├─ hyper escreve os bytes no TokioIo (TCP stream)
  │
  └─ Cliente recebe a resposta.
     A task do tokio::spawn continua viva para a próxima
     request na mesma conexão (HTTP keep-alive), ou encerra.
```

---

## FASE 4 — Cenários Alternativos

### 4.1 Rota Não Encontrada

```
POST /nonexistent
  │
  ├─ find_route("POST", "/nonexistent") → None
  ├─ method != "OPTIONS" e method != "HEAD"
  └─ handle_not_found("/nonexistent")
     │
     │  (src/server/router/core.rs linhas 291-307)
     │
     ├─ if let Some(_handler) = &self.not_found_handler {
     │      // Cria ctx mas NUNCA chama _handler ⚠ BUG
     │      let _ctx = MurRequestContext::new(...);
     │  }
     │
     └─ SEMPRE retorna:
        404 {"error":"Not Found","message":"No route found for path: /nonexistent","status":404}
```

### 4.2 Guard Rejeita

```
POST /admin/settings  (não é #[public], requer autenticação)
  │
  ├─ find_route → match
  ├─ collect_body → Ok
  ├─ MurRequestContext::new(...)
  ├─ execute_handler:
  │   ├─ ctx.with_access_control({ is_public: false, allowed_roles: {"admin"} })
  │   ├─ global_guards[0] = AuthGuard
  │   │   AuthGuard.can_activate(&ctx):
  │   │     ctx.is_public_route() → false
  │   │     ctx.bearer_token() → None (sem Authorization header)
  │   │     → return false
  │   │
  │   └─ AuthGuard.rejection_response():
  │      403 {"error":"Forbidden","message":"Access denied!"}
  │
  └─ Retorna 403. Handler NUNCA executa.
     Interceptors before/after NUNCA executam.
```

### 4.3 Interceptor Before Falha

```
POST /users
  │
  ├─ Guards → todos aprovam
  ├─ global_interceptors[0].before(&ctx):
  │   → Err(MurError::Unauthorized("Token expired"))
  │
  └─ handle_error(MurError::Unauthorized("Token expired"))
     → 401 {"error":"Unauthorized: Token expired","status":401,"kind":"unauthorized"}

     Handler NUNCA executa.
     Interceptors after NUNCA executam.
```

### 4.4 JSON Inválido no Body

```
POST /users  com body: "isso não é json"
  │
  ├─ Guards → aprovam
  ├─ Interceptors before → Ok
  ├─ (route.handler)(ctx):
  │   │
  │   └─ Extração MurJson<CreateUserDto>:
  │        ctx.json() → serde_json::from_slice(b"isso não é json")
  │        → Err(MurError::BadRequest("Invalid JSON: expected value at line 1..."))
  │        → return MurResponder::error("Failed to parse JSON: ...")
  │          → 400 Bad Request "Failed to parse JSON: Bad request: Invalid JSON: ..."
  │
  ├─ result = Err(MurError) OU Ok(400 response)
  │  // Depende: MurResponder::error retorna Ok(Response com status 400)
  │  // Então result = Ok(Response { status: 400, body: "Failed to parse JSON: ..." })
  │
  ├─ Interceptors after EXECUTAM (porque result é Ok)
  │
  └─ Retorna 400 para o cliente.
```

### 4.5 Service Lança Erro

```
POST /users  com body JSON válido
  │
  ├─ Guards → aprovam
  ├─ Interceptors before → Ok
  ├─ (route.handler)(ctx):
  │   ├─ MurJson extração → Ok
  │   └─ controller.create_user(body).await
  │      └─ self.user_service.create(dto).await
  │         └─ diesel::insert_into(...).execute(&mut conn)
  │            → Err(diesel::UniqueViolation)
  │            → Err(MurError::Custom(409, "duplicate key value..."))
  │            // O ? propaga para o handler
  │         → handler retorna Err(MurError::Custom(409, ...))
  │
  ├─ result = Err(MurError::Custom(409, ...))
  ├─ response = Err(MurError::Custom(409, ...))
  │
  ├─ Interceptors after EXECUTAM com a response Err
  │  // interceptor.after(&ctx, Err(e)) → pode transformar ou propagar
  │
  └─ match response {
         Err(e) => self.handle_error(e)
     }
     → exception_filters / error_handler / fallback
     → 409 {"error":"Error 409: duplicate key value...","status":409,"kind":"custom"}
```

### 4.6 HEAD para rota GET existente

```
HEAD /users
  │
  ├─ find_route("HEAD", "/users") → None
  │  // Nenhuma rota HEAD registrada
  │
  ├─ method == "HEAD" → tenta find_route("GET", "/users")
  │  → Some((route_get, params))
  │
  ├─ collect_body, MurRequestContext::new(...)
  │
  └─ execute_handler(route_get, ctx)
     // Executa o handler GET normalmente
     // hyper automaticamente descarta o body na response HEAD
```

### 4.7 OPTIONS (preflight CORS)

```
OPTIONS /users
  │
  ├─ find_route("OPTIONS", "/users") → None
  │  // Nenhuma rota OPTIONS registrada explicitamente
  │
  ├─ method == "OPTIONS"
  │
  └─ handle_options("/users")
     │  (src/server/router/core.rs linhas 309-330)
     │
     ├─ Itera TODAS as routes_by_method
     │  Para cada (method, routes):
     │    Se alguma route.pattern.match_path("/users") → adiciona method
     │  → methods = ["GET", "POST"]  (por exemplo)
     │
     ├─ allow = "GET, POST"
     │  // ⚠ Se nenhuma rota match: retorna TODOS os métodos (bug)
     │
     └─ 204 No Content
        Headers:
          Allow: GET, POST
          Access-Control-Allow-Methods: GET, POST
          Access-Control-Allow-Headers: Content-Type, Authorization
          Access-Control-Max-Age: 86400

        // ⚠ Note: como middleware não executa, MurCors nunca
        //    adiciona Access-Control-Allow-Origin aqui.
        //    Este handler de OPTIONS é hardcoded e limitado.
```

---

## FASE 5 — Diagrama de Dependências Completo

```
                    ┌──────────────────────────────────┐
                    │           MurInjects              │
                    │  HashMap<TypeId, Arc<Injectable>> │
                    │  Ex: { DbPool, RedisPool, ... }  │
                    └──────────┬───────────────────────┘
                               │ usado por
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                        MurModule                                 │
│  services_with_injects(injects, container):                      │
│    → MurServiceFactory::create(injects, container)               │
│    → Resolução topológica de dependências                        │
│  controllers_with_injects(injects, container):                   │
│    → MurControllerFactory::create(injects, container)            │
└───────────┬──────────────────────────────────────┬──────────────┘
            │ produz services                      │ produz controllers
            ▼                                      ▼
┌───────────────────────┐            ┌──────────────────────────────┐
│  MurServiceContainer   │            │  MurController (Arc)          │
│  HashMap<TypeId,       │◄───────────│  fn routes(self: Arc<Self>,   │
│    Arc<dyn MurService>>│  consulta   │    container) -> Vec<RouteDef>│
│                        │            │  Cada RouteDef contém:        │
│  Armazena:             │            │    - method: "POST"           │
│   - UserService        │            │    - path: "/users"           │
│   - AuthService        │            │    - handler: closure que:    │
│   - DbPool (via inject)│            │        1. extrai params do ctx│
│   - etc.               │            │        2. chama self.method() │
└──────────┬─────────────┘            └──────────────┬───────────────┘
           │                                         │
           │ Arc compartilhado em todas as requests   │ registrado no router
           ▼                                         ▼
┌──────────────────────────────────────────────────────────────────┐
│                          MurRouter                                │
│                                                                   │
│  routes_by_method: HashMap<String, Vec<MurRouteEntry>>            │
│    "GET"  → [ entry(/users, handler), entry(/users/:id, handler) ]│
│    "POST" → [ entry(/users, handler) ]                            │
│    ...                                                            │
│                                                                   │
│  global_guards: Vec<Arc<dyn MurGuard>>                            │
│  global_interceptors: Vec<Arc<dyn MurInterceptor>>                │
│  global_middleware: Vec<Arc<dyn MurMiddleware>>  ⚠ NUNCA POPULADO │
│  exception_filters: Vec<Arc<dyn MurExceptionFilter>>              │
│  container: Arc<MurServiceContainer>                              │
└──────────────────────────────┬───────────────────────────────────┘
                               │
                               │ Arc::new(router) → MurServerRunner
                               ▼
┌──────────────────────────────────────────────────────────────────┐
│                       MurServerRunner                             │
│                                                                   │
│  router: Arc<MurRouter>        ← compartilhado entre todas tasks  │
│  config: MurServerConfig                                          │
│  modules: Vec<Box<dyn MurModule>>                                 │
│  injects: MurInjects                                              │
│  on_startup / on_shutdown hooks                                   │
│  tls_acceptor: Option<MurTlsAcceptor>                             │
│                                                                   │
│  .run().await:                                                    │
│    TcpListener::bind → loop { accept → spawn(handle) }            │
└──────────────────────────────────────────────────────────────────┘
```

---

## FASE 6 — Resumo do Pipeline (Ordem Exata)

```
REQUEST TCP chega
  │
  ▼
TcpListener.accept()
  │
  ▼
tokio::spawn (nova task)
  │
  ▼
hyper http1::Builder.serve_connection()
  │  Parseia HTTP/1.1 → Request<Incoming>
  ▼
router.handle(req)
  │
  ├─ req.into_parts() → (Parts, body stream)
  ├─ method = "POST", path = "/users"
  ├─ find_route("POST", "/users") → Some(route, params)
  ├─ collect_body(body) → Some(Bytes)          ⚠ sem limite de tamanho
  ├─ MurRequestContext::new(parts, body, params, container)
  │
  ▼
execute_handler(route, ctx)
  │
  ├─ 1. ctx.with_access_control()              ← anexa is_public + roles
  ├─ 2. global_guards.can_activate()           ← pode rejeitar (403)
  ├─ 3. route.guards.can_activate()            ← pode rejeitar (403)
  ├─ 4. global_interceptors.before()           ← pode rejeitar (Err)
  ├─ 5. route.interceptors.before()            ← pode rejeitar (Err)
  │
  ├─ 6. ★ (route.handler)(ctx) ★               ← CLOSURE DO MACRO
  │     │
  │     ├─ Extrai parâmetros (MurJson, Param, MurQuery, etc.)
  │     │  Se falha → retorna 400 Bad Request
  │     │
  │     ├─ controller.method(params).await
  │     │  └─ self.service.do_work().await
  │     │     └─ usa injects (db, cache, etc.)
  │     │
  │     └─ Retorna MurRes (Ok ou Err)
  │
  ├─ 7. route.interceptors.after()  (REVERSO)  ← pode modificar response
  ├─ 8. global_interceptors.after() (REVERSO)  ← pode modificar response
  │
  ├─ 9. Se Ok(response) → retorna
  │     Se Err(error):
  │       ├─ exception_filters.catch()
  │       ├─ error_handler()
  │       └─ fallback: error.into_response()
  │
  ▼
hyper serializa Response → bytes TCP → cliente recebe

  ⚠ NUNCA EXECUTADOS (registrados mas ignorados):
     - global_middleware (CORS, Rate Limit, Security Headers, Compression, Timeout)
     - config.body_limit
     - config.keep_alive_timeout
     - config.http2
     - not_found_handler (criado mas não chamado)
```
