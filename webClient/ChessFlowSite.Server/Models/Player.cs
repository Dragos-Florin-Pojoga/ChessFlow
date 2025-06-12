namespace ChessFlowSite.Server.Models
{
    public class Player
    {
        public string ConnectionId { get; set; }
        public string Username { get; set; }
        public int Elo {  get; set; }
        public bool isGuest { get; set; }

        public string Format { get; set; }

        public string? UserId { get; set; }
    }
}
