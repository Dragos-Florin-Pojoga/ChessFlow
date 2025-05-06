using Microsoft.AspNetCore.Identity;
using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace ChessFlowSite.Server.Models
{
    public class ApplicationUser : IdentityUser
    {
        [MinLength(5, ErrorMessage ="Username must have at least 5 chracters")]
        [MaxLength(31, ErrorMessage = "Username can have at most 31 characters")]
        public string Name { get; set; }
        public int Elo {  get; set; }

        public bool isBanned { get; set; } = false;

        public virtual ICollection<Game?>? GamesAsWhite { get; set; }
        public virtual ICollection<Game?>? GamesAsBlack { get; set; }
        public virtual ICollection<Report?>? ReportsRecieved { get; set; }
        public virtual ICollection<Report?>? ReportsIssued { get; set; }
        public virtual ICollection<Ban?>? BansReceieved { get; set; }
        public virtual ICollection<Ban?>? BansIssued { get; set; }

        [NotMapped]
        public ICollection<Game> Games => [.. GamesAsWhite, .. GamesAsBlack];
    }
}
