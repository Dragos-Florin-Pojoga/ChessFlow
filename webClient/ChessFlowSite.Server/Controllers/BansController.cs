using ChessFlowSite.Server.Models;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using ReactApp1.Server.Data;
using System.ComponentModel.DataAnnotations.Schema;

namespace ChessFlowSite.Server.Controllers
{
    [Route("api/[controller]")]
    [ApiController]
    public class BansController : Controller
    {
        private readonly ApplicationDbContext _db;
        private readonly UserManager<ApplicationUser> _userManager;

        public BansController(ApplicationDbContext db, UserManager<ApplicationUser> userManager)
        {
            _db = db;
            _userManager = userManager;
        }

        [HttpPost("create")]
        [Authorize(Roles = "Admin")]
        public async Task<IActionResult> Create([FromBody] BanModel model) {
            var banned = _db.ApplicationUsers.FirstOrDefault(u => u.Name == model.BannedName);
            var issuer = await _userManager.GetUserAsync(User);

            if (issuer.Name != model.IssuerName || banned == null || banned == issuer)
            {
                return BadRequest(new { errors = new[] { new { code = "InvalidUsers", description = "Banned and/or issuer users are invalid" } } });
            }
            if (banned.isBanned) {
                return BadRequest(new { errors = new[] { new { code = "AlreadyBanned", description = "User is already banned" } } });
            }
            Report? report = null;
            if (model.ReportID != null) {
                report = _db.Reports.Include(r => r.Reported).FirstOrDefault(r => r.Id == model.ReportID);
                if (report == null || report.Reported.Id != banned.Id) {
                    return BadRequest(new { errors = new[] { new { code = "InvalidReport", description = "Report is invalid" } } });
                }
            }
            if (model.Reason.Length > 511)
            {
                ModelState.AddModelError("ContentTooLong", "Content cannot be longer than 511 characters.");
            }
            if (model.Reason.Length < 1)
            {
                ModelState.AddModelError("ContentTooShort", "Reason must be at least 1 character");
            }
            if (!ModelState.IsValid)
            {
                var errorList = ModelState.Where(ms => ms.Value.Errors.Count > 0).SelectMany(kvp => kvp.Value.Errors.Select(e => new
                {
                    code = kvp.Key,
                    description = e.ErrorMessage
                })).ToArray();
                return BadRequest(new { errors = errorList });
            }
            Ban ban = new Ban();
            ban.Banned = banned;
            ban.Issuer = issuer;
            ban.Report = report;
            ban.Reason = model.Reason;
            ban.Permanent = model.Permanent;
            ban.BannedAt = DateTime.UtcNow;
            ban.BannedUntil = model.EndDate;

            _db.Bans.Add(ban);
            banned.isBanned = true;
            await _db.SaveChangesAsync();
            return Ok();
        }

        [HttpPost("unban")]
        [Authorize(Roles = "Admin")]
        public async Task<IActionResult> Unban([FromBody] UnbanModel model)
        {
            var banned = _db.ApplicationUsers.FirstOrDefault(u => u.Name == model.BannedName);
            var issuer = await _userManager.GetUserAsync(User);

            if (issuer.Name != model.IssuerName || banned == null || banned == issuer)
            {
                return BadRequest(new { errors = new[] { new { code = "InvalidUsers", description = "Banned and/or issuer users are invalid" } } });
            }
            if (!banned.isBanned)
            {
                return BadRequest(new { errors = new[] { new { code = "AlreadyBanned", description = "User is not banned" } } });
            }
            if (!ModelState.IsValid)
            {
                var errorList = ModelState.Where(ms => ms.Value.Errors.Count > 0).SelectMany(kvp => kvp.Value.Errors.Select(e => new
                {
                    code = kvp.Key,
                    description = e.ErrorMessage
                })).ToArray();
                return BadRequest(new { errors = errorList });
            }
            Ban latestBan = _db.Bans.FirstOrDefault(b => b.BannedId == banned.Id && (b.Permanent == true || DateTime.Compare((DateTime)b.BannedUntil, DateTime.UtcNow) >= 0));
            if (latestBan != null) {
                latestBan.BannedUntil = DateTime.UtcNow;
                latestBan.Permanent = false;
            }
            else return BadRequest(new { errors = new[] { new { code = "Why", description = "Ugh" } } });
            banned.isBanned = false;
            await _db.SaveChangesAsync();
            return Ok();
        }

        [HttpGet("index")]
        [Authorize(Roles = "Admin")]
        public async Task<IActionResult> Index([FromQuery] BanIndexModel model)
        {
            if (model.Page <= 0 || model.PageSize <= 0)
                return BadRequest(new { errors = new[] { new { code = "InvalidPagination", description = "Page and pageSize must be greater than zero." } } });

            var query = _db.Bans
                .Include(r => r.Banned)
                .Include(r => r.Issuer)
                .Include(r => r.Report)
                .AsQueryable();

            // Filtering
            if (!string.IsNullOrWhiteSpace(model.BannedName))
            {
                query = query.Where(r => r.Banned.Name == model.BannedName);
            }

            if (model.showActiveOnly) {
                query = query.Where(b => b.Permanent == true || DateTime.Compare((DateTime)b.BannedUntil, DateTime.UtcNow) >= 0);
            }

            // Sorting
            query = (model.SortType.ToLower(), model.IsAscending) switch
            {
                ("banstart", true) => query.OrderBy(r => r.BannedAt),
                ("banstart", false) => query.OrderByDescending(r => r.BannedAt),
                ("id", true) => query.OrderBy(r => r.Id),
                _ => query.OrderByDescending(r => r.Id), // Default
            };

            // Pagination
            var totalItems = await query.CountAsync();
            var lastPage = Math.Ceiling((double)totalItems / model.PageSize);
            var reports = await query
                .Skip((model.Page - 1) * model.PageSize)
                .Take(model.PageSize)
                .ToListAsync();

            var result = new
            {
                lastPage = lastPage,
                items = reports.Select(r => new {
                    banID = r.Id,
                    bannedName = r.Banned?.Name,
                    issuerName = r.Issuer?.Name,
                    reportID = r.ReportId,
                    reason = r.Reason,
                    start = r.BannedAt,
                    end = r.BannedUntil,
                    currentlyBanned = r.Banned?.isBanned
                })
            };

            return Ok(result);
        }
    }
}

public class BanModel {
    public string BannedName { get; set; }
    public string IssuerName { get; set; }
    public string Reason { get; set; }
    public bool Permanent {  get; set; }
    public DateTime? EndDate { get; set; }
    public int? ReportID { get; set; }
}

public class UnbanModel
{
    public string BannedName { get; set; }
    public string IssuerName { get; set; }
}

public class BanIndexModel
{
    public string? BannedName { get; set; }
    public int Page { get; set; } = 1;
    public int PageSize { get; set; } = 10;
    public string? SortType { get; set; }
    public bool IsAscending { get; set; } = false;
    public bool showActiveOnly { get; set; } = false;
}
