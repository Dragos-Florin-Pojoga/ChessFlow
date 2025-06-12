using ChessFlowSite.Server.Hubs;
using ChessFlowSite.Server.Models;
using ChessFlowSite.Server.Services;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.AspNetCore.Identity;
using Microsoft.Build.Experimental;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.FileProviders;
using Microsoft.AspNetCore.StaticFiles;
using Microsoft.Extensions.Options;
using Microsoft.IdentityModel.Tokens;
using Microsoft.OpenApi.Models;
using ReactApp1.Server.Data;
using Swashbuckle.AspNetCore.Filters;
using System.Security.Claims;
using System.Text;

namespace ChessFlowSite.Server
{
    public class Program
    {
        public static void Main(string[] args)
        {
            var builder = WebApplication.CreateBuilder(args);

            // Add services to the container.
            builder.Services.AddHttpsRedirection(options =>
            {
                options.HttpsPort = 7073;
            });

            var connectionString = builder.Configuration.GetConnectionString("ApplicationDbContextConnection");
            builder.Services.AddDbContextFactory<ApplicationDbContext>(options => options.UseSqlite(connectionString));


            builder.Services.AddAuthentication(options =>
            {
                options.DefaultAuthenticateScheme = JwtBearerDefaults.AuthenticationScheme;
                options.DefaultChallengeScheme = JwtBearerDefaults.AuthenticationScheme;
            }).AddJwtBearer(options =>
            {
                options.TokenValidationParameters = new
                TokenValidationParameters
                {
                    ValidateIssuer = true,
                    ValidateAudience = true,
                    ValidateLifetime = true,
                    ValidateIssuerSigningKey = true,
                    ValidIssuer = builder.Configuration["Jwt:Issuer"],
                    ValidAudience = builder.Configuration["Jwt:Audience"],
                    IssuerSigningKey = new SymmetricSecurityKey(Encoding.UTF8.GetBytes(builder.Configuration["Jwt:Key"]))
                };
                options.Events = new JwtBearerEvents
                {
                    OnMessageReceived = context =>
                    {
                        var accessToken = context.Request.Query["access_token"];

                        // If the request is for our SignalR hub...
                        var path = context.HttpContext.Request.Path;
                        if (!string.IsNullOrEmpty(accessToken) &&
                            (path.StartsWithSegments("/api/gamehub")))
                        {
                            // Read the token out of the query string
                            context.Token = accessToken;
                        }
                        return Task.CompletedTask;
                    }
                };
            });
            builder.Services.AddSwaggerGen(option =>
            {
                option.AddSecurityDefinition("Bearer", new OpenApiSecurityScheme
                {
                    In = ParameterLocation.Header,
                    Description = "Please enter a valid token",
                    Name = "Authorization",
                    Type = SecuritySchemeType.Http,
                    BearerFormat = "JWT",
                    Scheme = "Bearer"
                });
                option.AddSecurityRequirement(new OpenApiSecurityRequirement
                {
                    {
                        new OpenApiSecurityScheme
                        {
                            Reference = new OpenApiReference
                            {
                                Type=ReferenceType.SecurityScheme,
                                Id="Bearer"
                            }
                        },
                        new string[]{ }
                    }
                });
                option.SwaggerDoc("v1", new OpenApiInfo { Title = "ChessFlow WebAPI", Version = "v1" });

                option.ExampleFilters();
            });
            builder.Services.AddSwaggerExamplesFromAssemblyOf<Program>();

            builder.Services.AddSignalR();
            builder.Services.AddSingleton<GameManager>();

            builder.Services.AddAuthorization();
            builder.Services.AddIdentityApiEndpoints<ApplicationUser>().AddRoles<IdentityRole>().AddEntityFrameworkStores<ApplicationDbContext>();

            builder.Services.AddControllers();
            // Learn more about configuring Swagger/OpenAPI at https://aka.ms/aspnetcore/swashbuckle
            builder.Services.AddEndpointsApiExplorer();

            var app = builder.Build();

            Console.WriteLine($"ContentRootPath: {builder.Environment.ContentRootPath}");

            // Add COOP and COEP headers globally
            app.Use(async (context, next) =>
            {
                context.Response.Headers.Add("Cross-Origin-Embedder-Policy", "require-corp");
                context.Response.Headers.Add("Cross-Origin-Opener-Policy", "same-origin");
                await next();
            });

            app.UseDefaultFiles();
            app.UseStaticFiles();

            // Configure custom static file serving for engine files
            app.UseStaticFiles(new StaticFileOptions
            {
                FileProvider = new PhysicalFileProvider(
                    Path.GetFullPath(Path.Combine(builder.Environment.ContentRootPath, "../../engine/public"))),
                // RequestPath = "/ChessFlowEngine",
                RequestPath = "/public",
                ContentTypeProvider = new FileExtensionContentTypeProvider
                {
                    Mappings =
                    {
                        [".js"] = "application/javascript",
                        [".wasm"] = "application/wasm"
                    }
                },
                OnPrepareResponse = ctx =>
                {
                    var headers = ctx.Context.Response.Headers;
                    // headers.Add("Cross-Origin-Embedder-Policy", "require-corp");
                    // headers.Add("Cross-Origin-Opener-Policy", "same-origin");
                    headers.Add("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0");
                    headers.Add("Pragma", "no-cache");
                    headers.Add("Expires", "0");
                }
            });

            using (var scope = app.Services.CreateScope())
            {
                var services = scope.ServiceProvider;
                SeedData.Initialize(services);
            }



            // Configure the HTTP request pipeline.
            if (app.Environment.IsDevelopment())
            {
                app.UseSwagger();
                app.UseSwaggerUI();
            }

            app.UseHttpsRedirection();


            app.UseAuthentication();
            app.UseAuthorization();

            app.MapHub<GameHub>("/api/gamehub");


            app.MapControllers();

            app.MapFallbackToFile("/index.html");

            app.Run();
        }
    }
}
