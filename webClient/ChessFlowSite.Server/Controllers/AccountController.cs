using ChessFlowSite.Server.Models;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;
using Microsoft.IdentityModel.Tokens;
using ReactApp1.Server.Data;
using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Text;

namespace ChessFlowSite.Server.Controllers
{
    [Route("api/[controller]")]
    [ApiController]
    public class AccountController : ControllerBase
    {
        private readonly ApplicationDbContext _db;
        private readonly UserManager<ApplicationUser> _userManager;
        private readonly SignInManager<ApplicationUser> _signInManager;
        private readonly RoleManager<IdentityRole> _roleManager;
        private readonly IConfiguration _configuration;
        public AccountController(ApplicationDbContext db, UserManager<ApplicationUser> userManager, SignInManager<ApplicationUser> signInManager, RoleManager<IdentityRole> roleManager, IConfiguration configuration)
        {
            _db = db;
            _userManager = userManager;
            _signInManager = signInManager;
            _roleManager = roleManager;
            _configuration = configuration;
        }

        [HttpPost("register")]
        public async Task<IActionResult> Register([FromBody] RegModel model)
        {
            if (!ModelState.IsValid)
                return BadRequest(ModelState);

            if (_db.Users.Any(u => u.Name == model.Name)) {
                return BadRequest(new {errors = new[] { new { code = "UsernameTaken", description = "Username is already taken" } } });
            }

            var user = new ApplicationUser
            {
                UserName = model.Email,
                Email = model.Email,
                Elo = model.Elo,
                Name = model.Name,
            };
            var result = await _userManager.CreateAsync(user, model.Password);
            if (!result.Succeeded)
                return BadRequest(new {errors =  result.Errors });
            await _userManager.AddToRoleAsync(user, "User");
            //temp
            var emailToken = await _userManager.GenerateEmailConfirmationTokenAsync(user);
            await _userManager.ConfirmEmailAsync(user, emailToken);
            return Ok(new { Message = "User registered successfully" });
        }

        [HttpPost("login")]
        public async Task<IActionResult> Login([FromBody] AuthModel model)
        {
            var user = await _userManager.FindByEmailAsync(model.Email);
            if (user == null || !await _userManager.CheckPasswordAsync(user, model.Password))
                return Unauthorized(new { Message = "Invalid credentials" });


            await _signInManager.SignInAsync(user, isPersistent: false);

            var roles = await _userManager.GetRolesAsync(user);
            var roleClaims = roles.Select(role => new Claim(ClaimTypes.Role, role));
            var authClaims = new List<Claim>
                {
                    new Claim(ClaimTypes.Name, user.UserName),
                    new Claim(ClaimTypes.NameIdentifier, user.Id),
                    new Claim(JwtRegisteredClaimNames.Jti, Guid.NewGuid().ToString())
                }.Union(roleClaims);

            var token = new JwtSecurityToken(
                issuer: _configuration["Jwt:Issuer"],
                audience: _configuration["Jwt:Audience"],
                expires: DateTime.UtcNow.AddHours(1),
                claims: authClaims,
                signingCredentials: new SigningCredentials(new SymmetricSecurityKey(Encoding.UTF8.GetBytes(_configuration["Jwt:Key"])), SecurityAlgorithms.HmacSha256)
            );
            return Ok(new
            {
                Token = new JwtSecurityTokenHandler().WriteToken(token)
            });
        }

        [HttpPost("logout")]
        [Authorize]
        public async Task<IResult> Logout() {
            await _signInManager.SignOutAsync();
            return Results.Ok();
        }

        [HttpGet("pingauth")]
        [Authorize]
        public async Task<IResult> PingAuth() {
            Console.WriteLine("test");
            var user = await _userManager.GetUserAsync(User);
            var email = user.Email;
            var name = user.Name;
            return Results.Json(new { email = email, name = name });
        }
        [HttpGet("user/{username}")]
        //[Authorize]
        public async Task<IResult> GetInfo(string username)
        {
            ApplicationUser user = _db.Users.FirstOrDefault(x => x.Name == username);
            if (user == null)
                return Results.NotFound(new { Message = "User not found" });

            return Results.Json(new
            {
                name =user.Name,
                elo = user.Elo
            });
        }

    }
}

public class AuthModel
{
    public string Email { get; set; }
    public string Password { get; set; }
}

public class RegModel
{
    public string Email { get; set; }
    public string Password { get; set; }

    public string Name { get; set; }
    
    public int Elo { get; set; }
}
