using ChessFlowSite.Server.Models;
using Microsoft.AspNetCore.Identity.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore;
using System.Reflection.Emit;

namespace ReactApp1.Server.Data
{
    public class ApplicationDbContext : IdentityDbContext<ApplicationUser>
    {
        public ApplicationDbContext(DbContextOptions<ApplicationDbContext> options)
            : base(options)
        {
        }

        public DbSet<ApplicationUser> ApplicationUsers { get; set; }
        public DbSet<Game> Games { get; set; }
        public DbSet<Report> Reports { get; set; }
        public DbSet<Ban> Bans { get; set; }

        protected override void OnModelCreating(ModelBuilder builder)
        {
            base.OnModelCreating(builder);
            builder.Entity<ApplicationUser>().HasIndex(u => u.Name).IsUnique();

            builder.Entity<Report>()
            .HasOne(b => b.Reported)
            .WithMany(u => u.ReportsIssued)
            .HasForeignKey(b => b.ReportedId)
            .OnDelete(DeleteBehavior.Restrict);

            builder.Entity<Report>()
            .HasOne(b => b.Reportee)
            .WithMany(u => u.ReportsRecieved)
            .HasForeignKey(b => b.ReporteeId)
            .OnDelete(DeleteBehavior.Restrict);

            builder.Entity<Ban>()
            .HasOne(b => b.Banned)
            .WithMany(u => u.BansReceieved)
            .HasForeignKey(b => b.BannedId)
            .OnDelete(DeleteBehavior.Restrict);

            builder.Entity<Ban>()
            .HasOne(b => b.Issuer)
            .WithMany(u => u.BansIssued)
            .HasForeignKey(b => b.IssuerId)
            .OnDelete(DeleteBehavior.Restrict);

            builder.Entity<Game>()
            .HasOne(g => g.PlayerWhite)
            .WithMany(u => u.GamesAsWhite)
            .HasForeignKey(g => g.PlayerWhiteId)
            .OnDelete(DeleteBehavior.Restrict);

            builder.Entity<Game>()
            .HasOne(g => g.PlayerBlack)
            .WithMany(u => u.GamesAsBlack)
            .HasForeignKey(g => g.PlayerBlackId)
            .OnDelete(DeleteBehavior.Restrict);

        }
    }
}
