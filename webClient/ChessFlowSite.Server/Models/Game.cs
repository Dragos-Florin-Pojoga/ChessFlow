using System.ComponentModel.DataAnnotations;

namespace ChessFlowSite.Server.Models
{
    public class Game
    {
        [Key]
        public int Id { get; set; }

        public string? PlayerWhiteId { get; set; }
        public virtual ApplicationUser? PlayerWhite { get; set; }
        
        public string? PlayerBlackId { get; set; }
        public virtual ApplicationUser? PlayerBlack { get; set;  }

        public string? GuestWhiteName { get; set; }
        public string? GuestBlackName { get; set; }

        public string? BotId { get; set; }

        public bool IsBotGame { get; set; }
        public bool IsRated { get; set; }

        public string Result { get; set; } = "InProgress";

        public int EloWhite {  get; set; }
        public int EloBlack { get; set; }

        public int? DeltaEloWhite { get; set; } = 0;
        public int? DeltaEloBlack { get; set; } = 0;

        public int MoveCount { get; set; } = 0;

        public string? Format { get; set; }

        public DateTime? StartTime { get; set; } = DateTime.UtcNow;
        public DateTime? EndTime { get; set; }

        public string? FinalFEN { get; set; }
        public string? PGN { get; set; }
    }
}
