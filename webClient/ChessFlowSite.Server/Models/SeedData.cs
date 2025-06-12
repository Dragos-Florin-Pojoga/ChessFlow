using Microsoft.AspNetCore.Identity;
using Microsoft.EntityFrameworkCore;
using ReactApp1.Server.Data;

namespace ChessFlowSite.Server.Models
{
    public static class SeedData
    {
        public static void Initialize(IServiceProvider serviceProvider)
        {

            using (var scope = serviceProvider.CreateScope())  // create a scope
            {
                var scopedProvider = scope.ServiceProvider;

                var factory = scopedProvider.GetRequiredService<IDbContextFactory<ApplicationDbContext>>();
                using (var context = factory.CreateDbContext())
                {
                    if (context.Roles.Any())
                    {
                        return;
                    }

                    context.Roles.AddRange(
                    new IdentityRole
                    {
                        Id = "2181910e-6cee-4a25-9b26-8bd4902b32b1",
                        Name = "User",
                        NormalizedName = "User".ToUpper()
                    },


                    new IdentityRole
                    {
                        Id = "fded05c9-a5b8-41a4-897a-bf4d4be0e15a",
                        Name = "Admin",
                        NormalizedName = "Admin".ToUpper()
                    }

                    );

                    var hasher = new PasswordHasher<ApplicationUser>();

                    context.Users.AddRange(
                        new ApplicationUser

                        {

                            Id = "8c735a3a-752c-4526-a3bc-c0e9d90c5db8",
                            // primary key
                            UserName = "admin@test.com",
                            EmailConfirmed = true,
                            NormalizedEmail = "ADMIN@TEST.COM",
                            Email = "admin@test.com",
                            NormalizedUserName = "ADMIN@TEST.COM",
                            PasswordHash = hasher.HashPassword(null, "Pass1!"),
                            Elo = 1000,
                            Name = "Admin"
                        },
                        new ApplicationUser

                        {

                            Id = "509c8f83-7200-422f-8336-5a25a7e67120",
                            // primary key
                            UserName = "user1@test.com",
                            EmailConfirmed = true,
                            NormalizedEmail = "USER1@TEST.COM",
                            Email = "user1@test.com",
                            NormalizedUserName = "USER1@TEST.COM",
                            PasswordHash = hasher.HashPassword(null, "User1!"),
                            Elo = 857,
                            Name = "User1"
                        },
                        new ApplicationUser

                        {
                            Id = "65a0083a-eb27-4662-beee-46b7dd578101",
                            // primary key
                            UserName = "user2@test.com",
                            EmailConfirmed = true,
                            NormalizedEmail = "USER2@TEST.COM",
                            Email = "user2@test.com",
                            NormalizedUserName = "USER2@TEST.COM",
                            PasswordHash = hasher.HashPassword(null, "User1!"),
                            Elo = 1015,
                            Name = "User2"
                        },
                        new ApplicationUser

                        {
                            Id = "9b477d7a-733d-4638-8b69-e54b05ffb770",
                            // primary key
                            UserName = "cheater@test.com",
                            EmailConfirmed = true,
                            NormalizedEmail = "CHEATER@TEST.COM",
                            Email = "cheater@test.com",
                            NormalizedUserName = "CHEATER@TEST.COM",
                            PasswordHash = hasher.HashPassword(null, "User1!"),
                            Elo = 1500,
                            Name = "Cheater",
                            isBanned = true
                        },
                        new ApplicationUser

                        {
                            Id = "28a54f22-baf4-413d-9e24-6c8aca587b15",
                            // primary key
                            UserName = "accused@test.com",
                            EmailConfirmed = true,
                            NormalizedEmail = "ACCUSED@TEST.COM",
                            Email = "accused@test.com",
                            NormalizedUserName = "ACCUSED@TEST.COM",
                            PasswordHash = hasher.HashPassword(null, "User1!"),
                            Elo = 1400,
                            Name = "Accused"
                        }
                    );

                    //  USER-ROLE ASSOCIATION
                    context.UserRoles.AddRange(
                    new IdentityUserRole<string>
                    {

                        RoleId = "fded05c9-a5b8-41a4-897a-bf4d4be0e15a",


                        UserId = "8c735a3a-752c-4526-a3bc-c0e9d90c5db8"
                    },

                    new IdentityUserRole<string>

                    {

                        RoleId = "2181910e-6cee-4a25-9b26-8bd4902b32b1",


                        UserId = "509c8f83-7200-422f-8336-5a25a7e67120"
                    },

                    new IdentityUserRole<string>

                    {

                        RoleId = "2181910e-6cee-4a25-9b26-8bd4902b32b1",


                        UserId = "65a0083a-eb27-4662-beee-46b7dd578101"
                    },

                    new IdentityUserRole<string>

                    {

                        RoleId = "2181910e-6cee-4a25-9b26-8bd4902b32b1",


                        UserId = "9b477d7a-733d-4638-8b69-e54b05ffb770"
                    },

                    new IdentityUserRole<string>

                    {

                        RoleId = "2181910e-6cee-4a25-9b26-8bd4902b32b1",


                        UserId = "28a54f22-baf4-413d-9e24-6c8aca587b15"
                    }

                    );
                    context.SaveChanges();
                }
            }
        }
    }
}
